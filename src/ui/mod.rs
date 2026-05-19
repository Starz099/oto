use eframe::egui;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use crate::app::{AppMessage, AudioProcess, UICommand};
use tray_icon::TrayIcon;
use crate::config::AppConfig;
use crate::discord::VcUser;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

mod mixer;
mod settings;
mod theme;

#[derive(PartialEq)]
pub enum AppScreen {
    Mixer,
    Settings,
}

#[derive(Clone, Copy, PartialEq)]
pub enum CustomKey {
    Egui(egui::Key),
    Ctrl,
    Alt,
    Shift,
}

pub struct MixerApp {
    pub(crate) initialized: bool,
    pub(crate) is_visible: bool,
    pub(crate) raw_sessions: Vec<AudioProcess>,
    pub(crate) sessions: Vec<AudioProcess>,
    pub(crate) rx: UnboundedReceiver<AppMessage>,
    pub(crate) tx_cmd: UnboundedSender<UICommand>,
    pub(crate) _tray_icon: TrayIcon,
    pub(crate) config: AppConfig,
    pub(crate) selected_index: usize,
    pub(crate) saved_volumes: HashMap<u32, f32>,
    pub(crate) discord_users: Vec<VcUser>,
    pub(crate) is_discord_accordion_open: bool,
    pub(crate) selected_discord_user_index: usize,
    pub(crate) ptt_enabled: Arc<AtomicBool>,
    pub(crate) is_ptt_held: bool,
    pub(crate) current_screen: AppScreen,
    pub(crate) recording_keybind: Option<String>,
    pub(crate) needs_restart: bool,
    pub(crate) original_hotkeys: crate::config::Hotkeys,
}

impl MixerApp {
    pub fn new(rx: UnboundedReceiver<AppMessage>, tx_cmd: UnboundedSender<UICommand>, tray_icon: TrayIcon, config: AppConfig, ptt_enabled: Arc<AtomicBool>) -> Self {
        let original_hotkeys = config.hotkeys.clone();
        Self {
            initialized: false,
            is_visible: true,
            raw_sessions: Vec::new(),
            sessions: Vec::new(),
            rx,
            tx_cmd,
            _tray_icon: tray_icon,
            config,
            selected_index: 0,
            saved_volumes: HashMap::new(),
            discord_users: Vec::new(),
            is_discord_accordion_open: false,
            selected_discord_user_index: 0,
            ptt_enabled,
            is_ptt_held: false,
            current_screen: AppScreen::Mixer,
            recording_keybind: None,
            needs_restart: false,
            original_hotkeys,
        }
    }

    pub(crate) fn parse_custom_key(&self, key_str: &str) -> Option<CustomKey> {
        match key_str.to_uppercase().as_str() {
            "A" => Some(CustomKey::Egui(egui::Key::A)),
            "B" => Some(CustomKey::Egui(egui::Key::B)),
            "C" => Some(CustomKey::Egui(egui::Key::C)),
            "D" => Some(CustomKey::Egui(egui::Key::D)),
            "E" => Some(CustomKey::Egui(egui::Key::E)),
            "F" => Some(CustomKey::Egui(egui::Key::F)),
            "G" => Some(CustomKey::Egui(egui::Key::G)),
            "H" => Some(CustomKey::Egui(egui::Key::H)),
            "I" => Some(CustomKey::Egui(egui::Key::I)),
            "J" => Some(CustomKey::Egui(egui::Key::J)),
            "K" => Some(CustomKey::Egui(egui::Key::K)),
            "L" => Some(CustomKey::Egui(egui::Key::L)),
            "M" => Some(CustomKey::Egui(egui::Key::M)),
            "N" => Some(CustomKey::Egui(egui::Key::N)),
            "O" => Some(CustomKey::Egui(egui::Key::O)),
            "P" => Some(CustomKey::Egui(egui::Key::P)),
            "Q" => Some(CustomKey::Egui(egui::Key::Q)),
            "R" => Some(CustomKey::Egui(egui::Key::R)),
            "S" => Some(CustomKey::Egui(egui::Key::S)),
            "T" => Some(CustomKey::Egui(egui::Key::T)),
            "U" => Some(CustomKey::Egui(egui::Key::U)),
            "V" => Some(CustomKey::Egui(egui::Key::V)),
            "W" => Some(CustomKey::Egui(egui::Key::W)),
            "X" => Some(CustomKey::Egui(egui::Key::X)),
            "Y" => Some(CustomKey::Egui(egui::Key::Y)),
            "Z" => Some(CustomKey::Egui(egui::Key::Z)),
            "0" => Some(CustomKey::Egui(egui::Key::Num0)),
            "1" => Some(CustomKey::Egui(egui::Key::Num1)),
            "2" => Some(CustomKey::Egui(egui::Key::Num2)),
            "3" => Some(CustomKey::Egui(egui::Key::Num3)),
            "4" => Some(CustomKey::Egui(egui::Key::Num4)),
            "5" => Some(CustomKey::Egui(egui::Key::Num5)),
            "6" => Some(CustomKey::Egui(egui::Key::Num6)),
            "7" => Some(CustomKey::Egui(egui::Key::Num7)),
            "8" => Some(CustomKey::Egui(egui::Key::Num8)),
            "9" => Some(CustomKey::Egui(egui::Key::Num9)),
            "F1" => Some(CustomKey::Egui(egui::Key::F1)),
            "F2" => Some(CustomKey::Egui(egui::Key::F2)),
            "F3" => Some(CustomKey::Egui(egui::Key::F3)),
            "F4" => Some(CustomKey::Egui(egui::Key::F4)),
            "F5" => Some(CustomKey::Egui(egui::Key::F5)),
            "F6" => Some(CustomKey::Egui(egui::Key::F6)),
            "F7" => Some(CustomKey::Egui(egui::Key::F7)),
            "F8" => Some(CustomKey::Egui(egui::Key::F8)),
            "F9" => Some(CustomKey::Egui(egui::Key::F9)),
            "F10" => Some(CustomKey::Egui(egui::Key::F10)),
            "F11" => Some(CustomKey::Egui(egui::Key::F11)),
            "F12" => Some(CustomKey::Egui(egui::Key::F12)),
            "BACKQUOTE" | "`" | "~" => Some(CustomKey::Egui(egui::Key::Backtick)),
            "CONTROLLEFT" | "CTRL" | "LEFTCONTROL" | "CONTROLRIGHT" | "RIGHTCONTROL" => Some(CustomKey::Ctrl),
            "ALT" | "ALTLEFT" | "ALTRIGHT" => Some(CustomKey::Alt),
            "SHIFTLEFT" | "LEFTSHIFT" | "SHIFTRIGHT" | "RIGHTSHIFT" => Some(CustomKey::Shift),
            "SPACE" => Some(CustomKey::Egui(egui::Key::Space)),
            "TAB" => Some(CustomKey::Egui(egui::Key::Tab)),
            "ENTER" => Some(CustomKey::Egui(egui::Key::Enter)),
            "ESCAPE" => Some(CustomKey::Egui(egui::Key::Escape)),
            "ARROWUP" | "UP" => Some(CustomKey::Egui(egui::Key::ArrowUp)),
            "ARROWDOWN" | "DOWN" => Some(CustomKey::Egui(egui::Key::ArrowDown)),
            "ARROWLEFT" | "LEFT" => Some(CustomKey::Egui(egui::Key::ArrowLeft)),
            "ARROWRIGHT" | "RIGHT" => Some(CustomKey::Egui(egui::Key::ArrowRight)),
            "PAGEUP" => Some(CustomKey::Egui(egui::Key::PageUp)),
            "PAGEDOWN" => Some(CustomKey::Egui(egui::Key::PageDown)),
            "HOME" => Some(CustomKey::Egui(egui::Key::Home)),
            "END" => Some(CustomKey::Egui(egui::Key::End)),
            "INSERT" => Some(CustomKey::Egui(egui::Key::Insert)),
            "DELETE" => Some(CustomKey::Egui(egui::Key::Delete)),
            _ => None,
        }
    }
}

impl eframe::App for MixerApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0] 
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                AppMessage::UpdateSessions(new_sessions) => {
                    let mut normalized_sessions = Vec::new();
                    let mut unique_sessions = Vec::new();
                    let mut seen_names = HashSet::new();
                    
                    for mut session in new_sessions {
                        // Clean up session name: chrome.exe -> Chrome
                        let cleaned_name = if session.name.to_lowercase().ends_with(".exe") {
                            let stem = &session.name[..session.name.len() - 4];
                            let mut chars = stem.chars();
                            match chars.next() {
                                None => stem.to_string(),
                                Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
                            }
                        } else {
                            session.name.clone()
                        };
                        session.name = cleaned_name;
                        normalized_sessions.push(session.clone());

                        if !seen_names.contains(&session.name) {
                            seen_names.insert(session.name.clone());
                            unique_sessions.push(session);
                        }
                    }

                    // Keep the raw cache normalized too so later volume updates match the
                    // same labels the UI shows.
                    self.raw_sessions = normalized_sessions;

                    let selected_name = self.sessions.get(self.selected_index).map(|s| s.name.clone());
                    self.sessions = unique_sessions;
                    
                    if let Some(name) = selected_name {
                        if let Some(new_index) = self.sessions.iter().position(|s| s.name == name) {
                            self.selected_index = new_index;
                        } else {
                            self.selected_index = self.selected_index.min(self.sessions.len().saturating_sub(1));
                        }
                    } else if self.sessions.is_empty() {
                        self.selected_index = 0;
                    }
                }
                AppMessage::ToggleOverlay => {
                    self.is_visible = !self.is_visible;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Visible(self.is_visible));
                    
                    if self.is_visible {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                        if let Some(monitor_size) = ctx.input(|i| i.viewport().monitor_size) {
                            let center_pos = egui::pos2(
                                (monitor_size.x - 550.0) / 2.0,
                                (monitor_size.y - 500.0) / 2.0,
                            );
                            ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(center_pos));
                        }
                    }
                }
                AppMessage::UpdateDiscordUsers(users) => {
                    self.discord_users = users;
                }
            }
        }

        if !self.initialized {
            let theme = theme::Theme::pastel_pink();
            theme::apply_theme(&ctx, &theme);

            if let Some(monitor_size) = ctx.input(|i| i.viewport().monitor_size) {
                let center_pos = egui::pos2(
                    (monitor_size.x - 550.0) / 2.0,
                    (monitor_size.y - 500.0) / 2.0,
                );
                ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(center_pos));
            }

            self.initialized = true;
        }


        let theme = theme::Theme::pastel_pink();
        let panel_frame = egui::Frame::new()
            .fill(theme.bg_dark) 
            .corner_radius(theme.corner_radius)
            .inner_margin(theme.spacing_outer)
            .stroke(egui::Stroke::new(1.0, theme.bg_selection));

        egui::CentralPanel::default().frame(panel_frame).show_inside(ui, |ui| {
            let app_rect = ui.max_rect();
            let title_bar_response = ui.interact(app_rect, ui.id().with("window_drag"), egui::Sense::drag());
            if title_bar_response.dragged() {
                ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
            }

            ui.horizontal(|ui| {
                ui.heading(egui::RichText::new("oto")
                    .color(theme.primary_pink)
                    .strong());
                
                ui.add_space(12.0);

                // PTT Indicator in Top Bar
                let ptt_enabled = self.ptt_enabled.load(std::sync::atomic::Ordering::Relaxed);
                let (status_text, status_color) = if !ptt_enabled {
                    ("PTT OFF", theme.text_dim)
                } else if self.is_ptt_held {
                    ("• LIVE", theme.primary_pink)
                } else {
                    ("READY", theme.text_accent)
                };

                egui::Frame::new()
                    .fill(status_color.gamma_multiply(0.1))
                    .stroke(egui::Stroke::new(1.0, status_color.gamma_multiply(0.5)))
                    .corner_radius(4)
                    .inner_margin(egui::Margin::symmetric(8, 2))
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new(status_text)
                            .color(status_color)
                            .size(12.0)
                            .strong());
                    });
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let icon = match self.current_screen {
                        AppScreen::Mixer => "⚙",
                        AppScreen::Settings => "⬅",
                    };
                    
                    let btn = ui.add(egui::Button::new(egui::RichText::new(icon).size(18.0))
                        .frame(false));
                    
                    if btn.clicked() {
                        self.current_screen = match self.current_screen {
                            AppScreen::Mixer => AppScreen::Settings,
                            AppScreen::Settings => AppScreen::Mixer,
                        };
                        self.recording_keybind = None;
                    }
                });
            });

            ui.add_space(theme.item_spacing);
            ui.add(egui::Separator::default().horizontal());
            ui.add_space(theme.item_spacing);

            match self.current_screen {
                AppScreen::Mixer => self.show_mixer_ui(ui),
                AppScreen::Settings => self.show_settings_ui(ui),
            }
        });
    }
}
