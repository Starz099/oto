use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::windows::named_pipe::{ClientOptions, NamedPipeClient};
use serde_json::json;
use std::time::SystemTime;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct VcUser {
    pub id: String,
    pub username: String,
    pub volume: u32,
    pub mute: bool,
}
// Discord IPC Opcodes
const OP_HANDSHAKE: u32 = 0;
const OP_FRAME: u32 = 1;
const IPC_PATH: &str = r"\\.\pipe\discord-ipc-0";
const DISCORD_CLIENT_ID: &str = "1505298148887630006";
const LOCAL_API_URL: &str = "https://raw-mixer-api-inlu.vercel.app/api/auth";

/// Helper function to pack JSON into Discord's strict binary format
async fn send_ipc_frame(pipe: &mut NamedPipeClient, opcode: u32, payload: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let payload_bytes = payload.as_bytes();
    let length = payload_bytes.len() as u32;

    // Write the Opcode (4 bytes, Little Endian)
    pipe.write_u32_le(opcode).await?;
    // Write the Length (4 bytes, Little Endian)
    pipe.write_u32_le(length).await?;
    
    // Write the actual JSON string payload
    pipe.write_all(payload_bytes).await?;
    
    Ok(())
}

/// Helper function to read Discord's binary response and unpack it to JSON
async fn read_ipc_frame(pipe: &mut NamedPipeClient) -> Result<(u32, String), Box<dyn std::error::Error + Send + Sync>> {
    // Read the Opcode (4 bytes)
    let opcode = pipe.read_u32_le().await?;    
    // Read the Length of the incoming JSON (4 bytes)
    let length = pipe.read_u32_le().await?;
    
    let mut buffer = vec![0; length as usize];
    pipe.read_exact(&mut buffer).await?;
    
    // Convert the bytes back into a readable String
    let payload = String::from_utf8(buffer)?;
    
    Ok((opcode, payload))
}

/// Connects to Discord's Named Pipe and performs the initial Handshake
pub async fn connect_to_discord() -> Result<NamedPipeClient, Box<dyn std::error::Error + Send + Sync>> {
    println!("Looking for Discord IPC socket...");
    
    // Try to open the Named Pipe
    let mut pipe = match ClientOptions::new().open(IPC_PATH) {
        Ok(client) => client,
        Err(e) => {
            return Err(format!("Could not connect to Discord. Is Discord running? Error: {}", e).into());
        }
    };

    println!("Socket found! Initiating Handshake...");

    // Prepare the specific Handshake payload
    let handshake = json!({
        "v": 1,
        "client_id": DISCORD_CLIENT_ID
    });

    // Send it using Opcode 0 (OP_HANDSHAKE)
    send_ipc_frame(&mut pipe, OP_HANDSHAKE, &handshake.to_string()).await?;

    // Wait for Discord to acknowledge our connection
    let (opcode, response) = read_ipc_frame(&mut pipe).await?;
    
    // Discord should reply with Opcode 1 (Frame) and a DISPATCH event containing our user info
    if opcode == OP_FRAME && response.contains("DISPATCH") {
        println!("Handshake successful! We are in.");
        Ok(pipe)
    } else {
        Err(format!("Handshake failed. Discord said: {}", response).into())
    }
}


/// Main entry point: Connects, asks for permission, and gets the token from Vercel/Express
pub async fn get_access_token() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    // Open the socket and perform the handshake
    let mut pipe = connect_to_discord().await?;

    // Generate a quick unique nonce using system time
    let nonce = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_millis()
        .to_string();

    // Prepare the exact AUTHORIZE payload
    let auth_payload = json!({
        "cmd": "AUTHORIZE",
        "args": {
            "client_id": DISCORD_CLIENT_ID,
            "scopes": ["rpc", "rpc.voice.write", "rpc.voice.read"] 
        },
        "nonce": nonce
    });

    println!("Triggering Discord OAuth popup on screen...");
    
    // Send it as a Frame (Opcode 1)
    send_ipc_frame(&mut pipe, OP_FRAME, &auth_payload.to_string()).await?;

    // Wait for the user to click "Authorize" and grab the response
    let (_opcode, response_string) = read_ipc_frame(&mut pipe).await?;
    let response_json: serde_json::Value = serde_json::from_str(&response_string)?;
    
    // Safety check: Did Discord give us an error instead?
    if response_json["evt"] == "ERROR" {
        return Err(format!("Discord denied access: {}", response_string).into());
    }

    // Extract the code
    let code = response_json["data"]["code"]
        .as_str()
        .ok_or("Failed to extract authorization code from Discord response")?;
        
    println!("Got Auth Code: {}. Verifying with Node server...", code);

    // Send the code to your Express server using reqwest
    let client = reqwest::Client::new();
    let res = client.post(LOCAL_API_URL)
        .json(&json!({ "code": code }))
        .send()
        .await?;

    if !res.status().is_success() {
        return Err(format!("Express Server returned error: {}", res.status()).into());
    }

    // Extract the final Access Token
    let token_data: serde_json::Value = res.json().await?;
    let access_token = token_data["access_token"]
        .as_str()
        .ok_or("No access_token found in server response")?
        .to_string();

    println!("Success! Locked in Access Token: {}", access_token);
    
    Ok(access_token)
}


/// Authenticates an open IPC socket using your saved token and returns your local User ID
pub async fn authenticate_socket(
    pipe: &mut tokio::net::windows::named_pipe::NamedPipeClient, 
    token: &str
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let nonce = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH)?.as_millis().to_string();
    
    let auth_payload = serde_json::json!({
        "cmd": "AUTHENTICATE",
        "args": {
            "access_token": token
        },
        "nonce": nonce
    });

    send_ipc_frame(pipe, OP_FRAME, &auth_payload.to_string()).await?;
    
    let (_, response) = read_ipc_frame(pipe).await?;
    let res_json: serde_json::Value = serde_json::from_str(&response)?;
    
    if res_json["evt"] == "ERROR" {
        return Err(format!("Socket Auth failed: {}", response).into());
    }
    
    // Extract your personal User ID from the auth response
    let current_user_id = res_json["data"]["user"]["id"]
        .as_str()
        .unwrap_or("")
        .to_string();

    Ok(current_user_id)
}

/// Fetches VC users using an ALREADY OPEN, persistent socket, filtering out the local user
pub async fn get_current_vc_users_persistent(
    pipe: &mut tokio::net::windows::named_pipe::NamedPipeClient,
    local_user_id: &str // NEW PARAMETER
) -> Result<Vec<VcUser>, Box<dyn std::error::Error + Send + Sync>> {
    let nonce = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH)?.as_millis().to_string();
    let payload = serde_json::json!({
        "cmd": "GET_SELECTED_VOICE_CHANNEL",
        "nonce": nonce
    });

    send_ipc_frame(pipe, OP_FRAME, &payload.to_string()).await?;
    
    loop {
        let (_, response) = read_ipc_frame(pipe).await?;
        let res_json: serde_json::Value = serde_json::from_str(&response)?;

        if res_json["cmd"] == "DISPATCH" { continue; }

        if res_json["data"].is_null() {
            return Ok(vec![]);
        }

        let mut users = Vec::new();
        if let Some(voice_states) = res_json["data"]["voice_states"].as_array() {
            for state in voice_states {
                let id = state["user"]["id"].as_str().unwrap_or("").to_string();
                
                // NEW: Skip adding the user to the list if it is YOU
                if id == local_user_id {
                    continue;
                }

                let username = state["user"]["username"].as_str().unwrap_or("Unknown").to_string();
                let volume = state["volume"].as_u64().unwrap_or(100) as u32;
                let mute = state["mute"].as_bool().unwrap_or(false);
                users.push(VcUser { id, username, volume, mute });
            }
        }
        return Ok(users);
    }
}


/// Updates user volume using an ALREADY OPEN, persistent socket
pub async fn set_user_voice_settings_persistent(
    pipe: &mut tokio::net::windows::named_pipe::NamedPipeClient, 
    user_id: &str, 
    volume: u32, 
    mute: bool
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let nonce = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH)?.as_millis().to_string();
    let payload = serde_json::json!({
        "cmd": "SET_USER_VOICE_SETTINGS",
        "args": {
            "user_id": user_id,
            "volume": volume,
            "mute": mute
        },
        "nonce": nonce
    });

    send_ipc_frame(pipe, OP_FRAME, &payload.to_string()).await?;
    
    loop {
        let (_, response) = read_ipc_frame(pipe).await?;
        let res_json: serde_json::Value = serde_json::from_str(&response)?;
        if res_json["cmd"] == "DISPATCH" { continue; }
        break;
    }

    Ok(())
}