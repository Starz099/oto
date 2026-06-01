use tokio::sync::mpsc;
use crate::app::{AppMessage, UICommand};
use crate::audio::wasapi::{WasapiManager, PersistentMic};
use crate::discord::DiscordClient;
use crate::config::AppConfig;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use crate::input::mapping::parse_key;
use crate::input::Action;

pub struct Backend {
    tx_ui: mpsc::UnboundedSender<AppMessage>,
    rx_cmd: mpsc::UnboundedReceiver<UICommand>,
    config: AppConfig,
    wasapi: WasapiManager,
    discord: DiscordClient,
    ptt_enabled: Arc<AtomicBool>,
    sessions_cache: Vec<crate::app::AudioProcess>,
}

impl Backend {
    pub fn new(
        tx_ui: mpsc::UnboundedSender<AppMessage>,
        rx_cmd: mpsc::UnboundedReceiver<UICommand>,
        config: AppConfig,
        ptt_enabled: Arc<AtomicBool>,
    ) -> Self {
        let wasapi = WasapiManager::new();
        let discord = DiscordClient::new(config.discord_client_id.clone().unwrap_or_default());
        
        Self {
            tx_ui,
            rx_cmd,
            config,
            wasapi,
            discord,
            ptt_enabled,
            sessions_cache: Vec::new(),
        }
    }

    pub async fn run(mut self) {
        let (tx_action, mut rx_action) = mpsc::unbounded_channel::<Action>();
        
        // Spawn Input Listener
        let config_clone = self.config.clone();
        let ptt_enabled_clone = self.ptt_enabled.clone();
        std::thread::spawn(move || {
            let overlay_key = parse_key(&config_clone.hotkeys.toggle_overlay);
            let ptt_toggle_key = parse_key(&config_clone.hotkeys.ptt_mode_toggle);
            let ptt_hold_key = parse_key(&config_clone.hotkeys.ptt_mic_hold);

            let mut is_ptt_held = false;
            let mut is_toggle_held = false;

            let _ = rdev::listen(move |event| {
                match event.event_type {
                    rdev::EventType::KeyPress(key) => {
                        if key == overlay_key {
                            let _ = tx_action.send(Action::ToggleOverlay);
                        }
                        if key == ptt_toggle_key {
                            if !is_toggle_held {
                                is_toggle_held = true;
                                let _ = tx_action.send(Action::TogglePttMode);
                            }
                        }
                        if key == ptt_hold_key {
                            if !is_ptt_held && ptt_enabled_clone.load(Ordering::Relaxed) {
                                is_ptt_held = true;
                                let _ = tx_action.send(Action::PttHold);
                            }
                        }
                    }
                    rdev::EventType::KeyRelease(key) => {
                        if key == ptt_toggle_key { is_toggle_held = false; }
                        if key == ptt_hold_key {
                            if is_ptt_held {
                                is_ptt_held = false;
                                let _ = tx_action.send(Action::PttRelease);
                            }
                        }
                    }
                    _ => {}
                }
            });
        });

        let mut mic = PersistentMic::new().expect("Failed to bind to mic");
        let mut mic_refresh = tokio::time::interval(Duration::from_secs(5));
        let mut audio_poll = tokio::time::interval(Duration::from_secs(1));
        let mut discord_poll = tokio::time::interval(Duration::from_secs(3));

        let mut discord_connected = false;
        let mut local_user_id = String::new();
        if let Some(token) = self.config.discord_access_token.clone() {
            if self.discord.connect().await.is_ok() {
                if let Ok(uid) = self.discord.authenticate(&token).await {
                    local_user_id = uid;
                    discord_connected = true;
                }
            }
        }

        if let Ok(sessions) = self.wasapi.get_active_sessions() {
            self.sessions_cache = sessions.clone();
            let _ = self.tx_ui.send(AppMessage::UpdateSessions(sessions));
        }

        loop {
            tokio::select! {
                Some(action) = rx_action.recv() => {
                    match action {
                        Action::ToggleOverlay => {
                            let _ = self.tx_ui.send(AppMessage::ToggleOverlay);
                        }
                        Action::TogglePttMode => {
                            let new_state = !self.ptt_enabled.load(Ordering::Relaxed);
                            self.ptt_enabled.store(new_state, Ordering::Relaxed);
                            let _ = mic.set_mute(new_state);
                        }
                        Action::PttHold => {
                            let _ = mic.set_mute(false);
                        }
                        Action::PttRelease => {
                            if self.ptt_enabled.load(Ordering::Relaxed) {
                                let _ = mic.set_mute(true);
                            }
                        }
                        _ => {}
                    }
                }
                _ = mic_refresh.tick() => {
                    let _ = mic.refresh();
                }
                _ = audio_poll.tick() => {
                    if let Ok(new_sessions) = self.wasapi.get_active_sessions() {
                        self.sessions_cache = new_sessions.clone();
                        let _ = self.tx_ui.send(AppMessage::UpdateSessions(new_sessions));
                    }
                }
                _ = discord_poll.tick() => {
                    if discord_connected {
                        if let Ok(users) = self.discord.get_vc_users(&local_user_id).await {
                            let _ = self.tx_ui.send(AppMessage::UpdateDiscordUsers(users));
                        }
                    }
                }
                Some(cmd) = self.rx_cmd.recv() => {
                    match cmd {
                        UICommand::SetProcessVolume { name, volume } => {
                            for session in &self.sessions_cache {
                                if session.name == name {
                                    let _ = self.wasapi.set_process_volume(session.pid, volume);
                                }
                            }
                        }
                        UICommand::SetDiscordUserVolume { user_id, volume, mute } => {
                            if discord_connected {
                                let _ = self.discord.set_user_voice_settings(&user_id, volume, mute).await;
                            }
                        }
                        UICommand::SetGlobalMicMute { muted } => {
                            let _ = mic.set_mute(muted);
                        }
                    }
                }
            }
        }
    }
}
