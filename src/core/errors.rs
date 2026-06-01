use thiserror::Error;

#[allow(dead_code)]
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Audio error: {0}")]
    Audio(#[from] AudioError),

    #[error("Discord error: {0}")]
    Discord(#[from] DiscordError),

    #[error("Config error: {0}")]
    Config(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

#[allow(dead_code)]
#[derive(Error, Debug)]
pub enum AudioError {
    #[error("COM error: {0}")]
    Com(#[from] windows::core::Error),

    #[error("Device not found")]
    DeviceNotFound,

    #[error("Session not found: {0}")]
    SessionNotFound(u32),
}

#[allow(dead_code)]
#[derive(Error, Debug)]
pub enum DiscordError {
    #[error("IPC error: {0}")]
    Ipc(String),

    #[error("Auth error: {0}")]
    Auth(String),

    #[error("Not connected")]
    NotConnected,
}
