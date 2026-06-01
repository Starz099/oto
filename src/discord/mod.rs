use tokio::net::windows::named_pipe::{ClientOptions, NamedPipeClient};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::SystemTime;
use crate::core::DiscordError;

pub mod ipc;
pub mod oauth;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct VcUser {
    pub id: String,
    pub username: String,
    pub volume: u32,
    pub mute: bool,
}

pub struct DiscordClient {
    pipe: Option<NamedPipeClient>,
    client_id: String,
}

const IPC_PATH: &str = r"\\.\pipe\discord-ipc-0";

impl DiscordClient {
    pub fn new(client_id: String) -> Self {
        Self {
            pipe: None,
            client_id,
        }
    }

    pub async fn connect(&mut self) -> Result<(), DiscordError> {
        let mut pipe = ClientOptions::new()
            .open(IPC_PATH)
            .map_err(|e| DiscordError::Ipc(format!("Could not connect to Discord: {}", e)))?;

        let handshake = json!({
            "v": 1,
            "client_id": self.client_id
        });

        ipc::send_frame(&mut pipe, ipc::OP_HANDSHAKE, &handshake.to_string()).await?;
        let (opcode, response) = ipc::read_frame(&mut pipe).await?;

        if opcode == ipc::OP_FRAME && response.contains("DISPATCH") {
            self.pipe = Some(pipe);
            Ok(())
        } else {
            Err(DiscordError::Ipc(format!("Handshake failed: {}", response)))
        }
    }

    pub async fn authenticate(&mut self, token: &str) -> Result<String, DiscordError> {
        let pipe = self.pipe.as_mut().ok_or(DiscordError::NotConnected)?;
        let nonce = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis()
            .to_string();

        let auth_payload = json!({
            "cmd": "AUTHENTICATE",
            "args": { "access_token": token },
            "nonce": nonce
        });

        ipc::send_frame(pipe, ipc::OP_FRAME, &auth_payload.to_string()).await?;
        let (_, response) = ipc::read_frame(pipe).await?;
        let res_json: serde_json::Value = serde_json::from_str(&response)
            .map_err(|e| DiscordError::Ipc(e.to_string()))?;

        if res_json["evt"] == "ERROR" {
            return Err(DiscordError::Auth(response));
        }

        let user_id = res_json["data"]["user"]["id"]
            .as_str()
            .ok_or_else(|| DiscordError::Ipc("No user id in auth response".to_string()))?
            .to_string();

        Ok(user_id)
    }

    pub async fn get_auth_code(&mut self) -> Result<String, DiscordError> {
        let pipe = self.pipe.as_mut().ok_or(DiscordError::NotConnected)?;
        let nonce = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis()
            .to_string();

        let auth_payload = json!({
            "cmd": "AUTHORIZE",
            "args": {
                "client_id": self.client_id,
                "scopes": ["rpc", "rpc.voice.write", "rpc.voice.read"]
            },
            "nonce": nonce
        });

        ipc::send_frame(pipe, ipc::OP_FRAME, &auth_payload.to_string()).await?;
        let (_, response_string) = ipc::read_frame(pipe).await?;
        let response_json: serde_json::Value = serde_json::from_str(&response_string)
            .map_err(|e| DiscordError::Ipc(e.to_string()))?;

        if response_json["evt"] == "ERROR" {
            return Err(DiscordError::Auth(response_string));
        }

        let code = response_json["data"]["code"]
            .as_str()
            .ok_or_else(|| DiscordError::Auth("No auth code in response".to_string()))?
            .to_string();

        Ok(code)
    }

    pub async fn get_vc_users(&mut self, local_user_id: &str) -> Result<Vec<VcUser>, DiscordError> {
        let pipe = self.pipe.as_mut().ok_or(DiscordError::NotConnected)?;
        let nonce = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis()
            .to_string();
            
        let payload = json!({
            "cmd": "GET_SELECTED_VOICE_CHANNEL",
            "nonce": nonce
        });

        ipc::send_frame(pipe, ipc::OP_FRAME, &payload.to_string()).await?;
        
        loop {
            let (_, response) = ipc::read_frame(pipe).await?;
            let res_json: serde_json::Value = serde_json::from_str(&response)
                .map_err(|e| DiscordError::Ipc(e.to_string()))?;

            if res_json["cmd"] == "DISPATCH" { continue; }
            if res_json["data"].is_null() { return Ok(vec![]); }

            let mut users = Vec::new();
            if let Some(voice_states) = res_json["data"]["voice_states"].as_array() {
                for state in voice_states {
                    let id = state["user"]["id"].as_str().unwrap_or("").to_string();
                    if id == local_user_id { continue; }

                    let username = state["user"]["username"].as_str().unwrap_or("Unknown").to_string();
                    let volume = state["volume"].as_u64().unwrap_or(100) as u32;
                    let mute = state["mute"].as_bool().unwrap_or(false);
                    users.push(VcUser { id, username, volume, mute });
                }
            }
            return Ok(users);
        }
    }

    pub async fn set_user_voice_settings(&mut self, user_id: &str, volume: u32, mute: bool) -> Result<(), DiscordError> {
        let pipe = self.pipe.as_mut().ok_or(DiscordError::NotConnected)?;
        let nonce = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis()
            .to_string();

        let payload = json!({
            "cmd": "SET_USER_VOICE_SETTINGS",
            "args": {
                "user_id": user_id,
                "volume": volume,
                "mute": mute
            },
            "nonce": nonce
        });

        ipc::send_frame(pipe, ipc::OP_FRAME, &payload.to_string()).await?;
        
        loop {
            let (_, response) = ipc::read_frame(pipe).await?;
            let res_json: serde_json::Value = serde_json::from_str(&response)
                .map_err(|e| DiscordError::Ipc(e.to_string()))?;
            if res_json["cmd"] == "DISPATCH" { continue; }
            break;
        }

        Ok(())
    }
}
