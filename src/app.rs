#[derive(Clone, Debug)]
pub struct AudioProcess {
    pub pid: u32,
    pub name: String,
    pub volume: f32,
}

pub enum AppMessage {
    UpdateSessions(Vec<AudioProcess>),
    ToggleOverlay,
}

pub enum UICommand {
    SetProcessVolume { pid: u32, volume: f32 },
}