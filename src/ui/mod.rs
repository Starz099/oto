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
                    self.raw_sessions = new_sessions.clone();
                    
                    let mut unique_sessions = Vec::new();
                    let mut seen_names = HashSet::new();
                    
                    for session in new_sessions {
                        if !seen_names.contains(&session.name) {
                            seen_names.insert(session.name.clone());
                            unique_sessions.push(session);
                        }
                    }

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
                                (monitor_size.x - 450.0) / 2.0,
                                (monitor_size.y - 350.0) / 2.0,
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
            let mut style = (*ctx.global_style()).clone();
            style.text_styles.insert(egui::TextStyle::Body, egui::FontId::new(16.0, egui::FontFamily::Proportional));
            style.text_styles.insert(egui::TextStyle::Button, egui::FontId::new(16.0, egui::FontFamily::Proportional));
            style.text_styles.insert(egui::TextStyle::Heading, egui::FontId::new(26.0, egui::FontFamily::Proportional));
            ctx.set_global_style(style);

            let mut visuals = egui::Visuals::dark();
            visuals.panel_fill = egui::Color32::TRANSPARENT;
            visuals.window_fill = egui::Color32::TRANSPARENT;
            visuals.selection.bg_fill = egui::Color32::from_rgb(180, 180, 180);
            visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(40, 40, 40);
            visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(60, 60, 60);
            visuals.widgets.active.bg_fill = egui::Color32::from_rgb(220, 220, 220);

            let no_shadow = egui::epaint::Shadow {
                offset: [0, 0],
                blur: 0,
                spread: 0,
                color: egui::Color32::TRANSPARENT,
            };
            visuals.window_shadow = no_shadow;
            visuals.popup_shadow = no_shadow;

            ctx.set_visuals(visuals);
            
            if let Some(monitor_size) = ctx.input(|i| i.viewport().monitor_size) {
                let center_pos = egui::pos2(
                    (monitor_size.x - 450.0) / 2.0,
                    (monitor_size.y - 350.0) / 2.0,
                );
                ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(center_pos));
            }
            
            self.initialized = true;
        }

        let panel_frame = egui::Frame::new()
            .fill(egui::Color32::from_rgb(15, 15, 15)) 
            .corner_radius(2.0)
            .inner_margin(24.0)
            .shadow(egui::epaint::Shadow { 
                offset: [0, 10],
                blur: 30, 
                spread: 0, 
                color: egui::Color32::from_black_alpha(220), 
            });

        egui::CentralPanel::default().frame(panel_frame).show_inside(ui, |ui| {
            let app_rect = ui.max_rect();
            let title_bar_response = ui.interact(app_rect, ui.id().with("window_drag"), egui::Sense::drag());
            if title_bar_response.dragged() {
                ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
            }

            ui.horizontal(|ui| {
                ui.heading(egui::RichText::new("Raw Mixer")
                    .color(egui::Color32::from_rgb(250, 250, 250)));
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let icon = match self.current_screen {
                        AppScreen::Mixer => "⚙",
                        AppScreen::Settings => "⬅",
                    };
                    if ui.button(egui::RichText::new(icon).size(20.0)).clicked() {
                        self.current_screen = match self.current_screen {
                            AppScreen::Mixer => AppScreen::Settings,
                            AppScreen::Settings => AppScreen::Mixer,
                        };
                        self.recording_keybind = None;
                    }
                });
            });

            match self.current_screen {
                AppScreen::Mixer => self.show_mixer_ui(ui),
                AppScreen::Settings => self.show_settings_ui(ui),
            }
        });
    }
}
