use eframe::egui;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use crate::app::{AppMessage, AudioProcess, UICommand};
use tray_icon::TrayIcon;
use crate::config::AppConfig;
use crate::discord::VcUser;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

pub struct MixerApp {
    initialized: bool,
    is_visible: bool,
    raw_sessions: Vec<AudioProcess>,
    sessions: Vec<AudioProcess>,
    rx: UnboundedReceiver<AppMessage>,
    tx_cmd: UnboundedSender<UICommand>,
    _tray_icon: TrayIcon,
    config: AppConfig,
    selected_index: usize,
    last_g_time: f64,
    saved_volumes: HashMap<u32, f32>,
    pub discord_users: Vec<VcUser>,
    pub is_discord_accordion_open: bool,
    pub selected_discord_user_index: usize,
    pub ptt_enabled: Arc<AtomicBool>,
}

impl MixerApp {
    pub fn new(rx: UnboundedReceiver<AppMessage>, tx_cmd: UnboundedSender<UICommand>, tray_icon: TrayIcon, config: AppConfig, ptt_enabled: Arc<AtomicBool>) -> Self {
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
            last_g_time: 0.0,
            saved_volumes: HashMap::new(),
            discord_users: Vec::new(),
            is_discord_accordion_open: false,
            selected_discord_user_index: 0,
            ptt_enabled,
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

        let mut hide_requested = false;
        let mut j_pressed = false;
        let mut k_pressed = false;
        let mut h_pressed = false;
        let mut l_pressed = false;
        let mut g_pressed = false;
        let mut m_pressed = false;
        let mut enter_pressed = false;
        let mut shift_pressed = false;
        let mut current_time = 0.0;

        ctx.input(|i| {
            current_time = i.time;
            shift_pressed = i.modifiers.shift;

            if i.key_pressed(egui::Key::Escape) {
                hide_requested = true;
            }
            
            for event in &i.events {
                if let egui::Event::Text(text) = event {
                    if text.contains('`') || text.contains('~') {
                        hide_requested = true;
                    }
                }
            }

            j_pressed = i.key_pressed(egui::Key::J);
            k_pressed = i.key_pressed(egui::Key::K);
            h_pressed = i.key_pressed(egui::Key::H);
            l_pressed = i.key_pressed(egui::Key::L);
            g_pressed = i.key_pressed(egui::Key::G);
            m_pressed = i.key_pressed(egui::Key::M);
            enter_pressed = i.key_pressed(egui::Key::Enter);
        });

        let is_discord_selected = self.sessions.get(self.selected_index)
            .map_or(false, |s| s.name.to_lowercase().contains("discord"));

        if enter_pressed && is_discord_selected {
            self.is_discord_accordion_open = true;
            self.selected_discord_user_index = 0;
        }

        if hide_requested {
            if self.is_discord_accordion_open {
                self.is_discord_accordion_open = false;
            } else {
                self.is_visible = false;
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
            }
        }

        let session_count = self.sessions.len();
        if session_count > 0 && self.is_visible {
            if j_pressed {
                if self.is_discord_accordion_open && !self.discord_users.is_empty() {
                    self.selected_discord_user_index = (self.selected_discord_user_index + 1).min(self.discord_users.len() - 1);
                } else {
                    self.selected_index = (self.selected_index + 1).min(session_count - 1);
                }
            }
            if k_pressed {
                if self.is_discord_accordion_open && !self.discord_users.is_empty() {
                    self.selected_discord_user_index = self.selected_discord_user_index.saturating_sub(1);
                } else {
                    self.selected_index = self.selected_index.saturating_sub(1);
                }
            }

            if m_pressed {
                if self.is_discord_accordion_open && !self.discord_users.is_empty() {
                    let user = &mut self.discord_users[self.selected_discord_user_index];
                    user.mute = !user.mute;
                    let _ = self.tx_cmd.send(UICommand::SetDiscordUserVolume {
                        user_id: user.id.clone(),
                        volume: user.volume,
                        mute: user.mute,
                    });
                } else {
                    let current_vol = self.sessions[self.selected_index].volume;
                    let new_vol = if current_vol > 0.0 { 0.0 } else { 100.0 };
                    let target_name = self.sessions[self.selected_index].name.clone();
                    
                    self.sessions[self.selected_index].volume = new_vol;
                    
                    for raw_session in &mut self.raw_sessions {
                        if raw_session.name == target_name {
                            let vol_to_set = if current_vol > 0.0 {
                                self.saved_volumes.insert(raw_session.pid, raw_session.volume);
                                0.0
                            } else {
                                self.saved_volumes.remove(&raw_session.pid).unwrap_or(100.0)
                            };
                            raw_session.volume = vol_to_set;
                            let _ = self.tx_cmd.send(UICommand::SetProcessVolume { pid: raw_session.pid, volume: vol_to_set });
                        }
                    }
                }
            }

            if g_pressed {
                if shift_pressed {
                    self.selected_index = session_count - 1;
                } else {
                    if current_time - self.last_g_time < 0.5 {
                        self.selected_index = 0;
                        self.last_g_time = 0.0;
                    } else {
                        self.last_g_time = current_time;
                    }
                }
            }

            if h_pressed || l_pressed {
                let step = if shift_pressed { 
                    self.config.settings.fast_step_percent 
                } else { 
                    self.config.settings.normal_step_percent 
                };
                
                if self.is_discord_accordion_open && !self.discord_users.is_empty() {
                    let user = &mut self.discord_users[self.selected_discord_user_index];
                    let mut new_vol = user.volume as f32;
                    if h_pressed { new_vol = (new_vol - step).max(0.0); }
                    if l_pressed { new_vol = (new_vol + step).min(200.0); }
                    user.volume = new_vol as u32;
                    
                    let _ = self.tx_cmd.send(UICommand::SetDiscordUserVolume {
                        user_id: user.id.clone(),
                        volume: user.volume,
                        mute: user.mute,
                    });
                } else {
                    let current_vol = self.sessions[self.selected_index].volume;
                    let mut new_vol = current_vol;

                    if h_pressed { new_vol = (current_vol - step).max(0.0); }
                    if l_pressed { new_vol = (current_vol + step).min(100.0); }

                    let target_name = self.sessions[self.selected_index].name.clone();
                    self.sessions[self.selected_index].volume = new_vol;
                    
                    for raw_session in &mut self.raw_sessions {
                        if raw_session.name == target_name {
                            raw_session.volume = new_vol;
                            self.saved_volumes.remove(&raw_session.pid);
                            let _ = self.tx_cmd.send(UICommand::SetProcessVolume { pid: raw_session.pid, volume: new_vol });
                        }
                    }
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

            ui.heading(egui::RichText::new("Raw Mixer")
                .color(egui::Color32::from_rgb(250, 250, 250)));
                
            ui.add_space(8.0);
            ui.add(egui::Separator::default().horizontal());

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                let mut ptt = self.ptt_enabled.load(std::sync::atomic::Ordering::Relaxed);
                
                if ui.checkbox(&mut ptt, egui::RichText::new("🎙 Global PTT (Hold Ctrl)").color(egui::Color32::from_rgb(200, 200, 200))).changed() {
                    self.ptt_enabled.store(ptt, std::sync::atomic::Ordering::Relaxed);
                    
                    if ptt {
                        let _ = self.tx_cmd.send(UICommand::SetGlobalMicMute { muted: true });
                    }
                }
            });
                                    
            ui.add_space(16.0);

            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    for (index, session) in self.sessions.iter_mut().enumerate() {
                        let is_discord = session.name.to_lowercase().contains("discord");
                        let is_selected = index == self.selected_index && !self.is_discord_accordion_open;
                        
                        let background_color = if is_selected {
                            egui::Color32::from_rgb(45, 45, 45) 
                        } else {
                            egui::Color32::TRANSPARENT
                        };

                        let text_color = if is_selected || (is_discord && self.is_discord_accordion_open && index == self.selected_index) {
                            egui::Color32::from_rgb(255, 255, 255)
                        } else {
                            egui::Color32::from_rgb(140, 140, 140)
                        };

                        let row_response = egui::Frame::new()
                            .fill(background_color)
                            .corner_radius(2.0)
                            .inner_margin(8.0)
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new(&session.name).color(text_color));
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if is_discord {
                                            let icon = if self.is_discord_accordion_open { "🔽" } else { "◀" };
                                            ui.label(egui::RichText::new(icon).color(text_color));
                                        }
                                        
                                        let slider_response = ui.add(egui::Slider::new(&mut session.volume, 0.0..=100.0).show_value(false));
                                        if slider_response.changed() {
                                            let target_name = session.name.clone();
                                            for raw_session in &mut self.raw_sessions {
                                                if raw_session.name == target_name {
                                                    raw_session.volume = session.volume;
                                                    self.saved_volumes.remove(&raw_session.pid);
                                                    let _ = self.tx_cmd.send(UICommand::SetProcessVolume { 
                                                        pid: raw_session.pid, 
                                                        volume: session.volume 
                                                    });
                                                }
                                            }
                                        }
                                    });
                                });
                            }).response;

                        if is_selected {
                            row_response.scroll_to_me(Some(egui::Align::Center));
                        }
                        
                        if is_discord && self.is_discord_accordion_open && index == self.selected_index {
                            ui.indent("discord_accordion", |ui| {
                                if self.discord_users.is_empty() {
                                    ui.add_space(4.0);
                                    ui.label(egui::RichText::new("Not in a Voice Channel").color(egui::Color32::from_rgb(100, 100, 100)));
                                } else {
                                    for (i, user) in self.discord_users.iter_mut().enumerate() {
                                        let is_user_selected = i == self.selected_discord_user_index;
                                        let user_bg = if is_user_selected { egui::Color32::from_rgb(55, 55, 55) } else { egui::Color32::TRANSPARENT };
                                        let user_text = if is_user_selected { egui::Color32::from_rgb(255, 255, 255) } else { egui::Color32::from_rgb(180, 180, 180) };
                                        
                                        ui.add_space(2.0);
                                        let user_row_response = egui::Frame::new()
                                            .fill(user_bg)
                                            .corner_radius(2.0)
                                            .inner_margin(6.0)
                                            .show(ui, |ui| {
                                                ui.horizontal(|ui| {
                                                    let mute_icon = if user.mute { "🔇" } else { "🔊" };
                                                    ui.label(egui::RichText::new(format!("{}  {}", mute_icon, user.username)).color(user_text));
                                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                        let mut vol_f32 = user.volume as f32;
                                                        let slider_response = ui.add(egui::Slider::new(&mut vol_f32, 0.0..=200.0).show_value(false));
                                                        user.volume = vol_f32 as u32;
                                                        
                                                        if slider_response.changed() {
                                                            let _ = self.tx_cmd.send(UICommand::SetDiscordUserVolume {
                                                                user_id: user.id.clone(),
                                                                volume: user.volume,
                                                                mute: user.mute,
                                                            });
                                                        }
                                                    });
                                                });
                                            }).response;

                                        if is_user_selected {
                                            user_row_response.scroll_to_me(Some(egui::Align::Center));
                                        }
                                    }
                                }
                            });
                        }
                        ui.add_space(6.0);
                    }
                });
        });
    }
}