use crate::discord::VcUser;

#[derive(Clone, Debug)]
pub struct AudioProcess {
    pub pid: u32,
    pub name: String,
    pub volume: f32,
}

pub enum AppMessage {
    UpdateSessions(Vec<AudioProcess>),
    ToggleOverlay,
    UpdateDiscordUsers(Vec<VcUser>),
}

pub enum UICommand {
    SetProcessVolume { pid: u32, volume: f32 },
    SetDiscordUserVolume { user_id: String, volume: u32, mute: bool },
}