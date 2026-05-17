use std::error::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::windows::named_pipe::{ClientOptions, NamedPipeClient};
use serde_json::json;
use std::time::SystemTime;

// Discord IPC Opcodes
const OP_HANDSHAKE: u32 = 0;
const OP_FRAME: u32 = 1;
const IPC_PATH: &str = r"\\.\pipe\discord-ipc-0";
const DISCORD_CLIENT_ID: &str = "1505298148887630006";
const LOCAL_API_URL: &str = "https://raw-mixer-api-inlu.vercel.app/api/auth";

/// Helper function to pack JSON into Discord's strict binary format
async fn send_ipc_frame(pipe: &mut NamedPipeClient, opcode: u32, payload: &str) -> Result<(), Box<dyn Error>> {
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
async fn read_ipc_frame(pipe: &mut NamedPipeClient) -> Result<(u32, String), Box<dyn Error>> {
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
async fn connect_to_discord() -> Result<NamedPipeClient, Box<dyn Error>> {
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
pub async fn get_access_token() -> Result<String, Box<dyn Error>> {
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