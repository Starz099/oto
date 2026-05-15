pub enum AppMessage {
    UpdateStarzVolume(f32), 
}

pub enum UICommand {
    SetProcessVolume { pid: u32, volume: f32 },
}