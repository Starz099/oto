use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::windows::named_pipe::NamedPipeClient;
use crate::core::DiscordError;

pub const OP_HANDSHAKE: u32 = 0;
pub const OP_FRAME: u32 = 1;

pub async fn send_frame(pipe: &mut NamedPipeClient, opcode: u32, payload: &str) -> Result<(), DiscordError> {
    let payload_bytes = payload.as_bytes();
    let length = payload_bytes.len() as u32;

    pipe.write_u32_le(opcode).await.map_err(|e| DiscordError::Ipc(e.to_string()))?;
    pipe.write_u32_le(length).await.map_err(|e| DiscordError::Ipc(e.to_string()))?;
    pipe.write_all(payload_bytes).await.map_err(|e| DiscordError::Ipc(e.to_string()))?;
    
    Ok(())
}

pub async fn read_frame(pipe: &mut NamedPipeClient) -> Result<(u32, String), DiscordError> {
    let opcode = pipe.read_u32_le().await.map_err(|e| DiscordError::Ipc(e.to_string()))?;
    let length = pipe.read_u32_le().await.map_err(|e| DiscordError::Ipc(e.to_string()))?;
    
    let mut buffer = vec![0; length as usize];
    pipe.read_exact(&mut buffer).await.map_err(|e| DiscordError::Ipc(e.to_string()))?;
    
    let payload = String::from_utf8(buffer).map_err(|e| DiscordError::Ipc(e.to_string()))?;
    
    Ok((opcode, payload))
}
