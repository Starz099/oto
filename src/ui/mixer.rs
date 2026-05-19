use eframe::egui;
use crate::ui::MixerApp;
use crate::app::UICommand;
use crate::ui::theme;

impl MixerApp {
    pub(crate) fn show_mixer_ui(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        let theme = theme::Theme::pastel_pink();
        
        let mut hide_requested = false;
        let mut nav_up = false;
        let mut nav_down = false;
        let mut vol_dec = false;
        let mut vol_inc = false;
        let mut jump_top = false;
        let mut jump_bottom = false;
        let mut mute_pressed = false;
        let mut accordion_open = false;
        let mut accordion_close = false;

        let ptt_mode_egui_key = self.parse_custom_key(&self.config.hotkeys.ptt_mode_toggle);
        let ptt_hold_egui_key = self.parse_custom_key(&self.config.hotkeys.ptt_mic_hold);

        let k_nav_up = self.parse_custom_key(&self.config.hotkeys.nav_up);
        let k_nav_down = self.parse_custom_key(&self.config.hotkeys.nav_down);
        let k_vol_dec = self.parse_custom_key(&self.config.hotkeys.vol_decrease);
        let k_vol_inc = self.parse_custom_key(&self.config.hotkeys.vol_increase);
        let k_fast_mod = self.parse_custom_key(&self.config.hotkeys.fast_modifier);
        let k_jump_top = self.parse_custom_key(&self.config.hotkeys.jump_top);
        let k_jump_bottom = self.parse_custom_key(&self.config.hotkeys.jump_bottom);
        let k_mute = self.parse_custom_key(&self.config.hotkeys.mute);
        let k_accordion_open = self.parse_custom_key(&self.config.hotkeys.accordion_open);
        let k_accordion_close = self.parse_custom_key(&self.config.hotkeys.accordion_close);

        let mut fast_mod_active = false;

        ctx.input(|i| {
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

            let check_key = |key_opt: Option<crate::ui::CustomKey>| -> bool {
                if let Some(crate::ui::CustomKey::Egui(k)) = key_opt {
                    i.key_pressed(k)
                } else {
                    false
                }
            };

            let is_modifier_down = |mod_opt: Option<crate::ui::CustomKey>| -> bool {
                match mod_opt {
                    Some(crate::ui::CustomKey::Ctrl) => i.modifiers.ctrl,
                    Some(crate::ui::CustomKey::Alt) => i.modifiers.alt,
                    Some(crate::ui::CustomKey::Shift) => i.modifiers.shift,
                    _ => false,
                }
            };

            nav_up = check_key(k_nav_up);
            nav_down = check_key(k_nav_down);
            vol_dec = check_key(k_vol_dec);
            vol_inc = check_key(k_vol_inc);
            fast_mod_active = is_modifier_down(k_fast_mod);
            jump_top = check_key(k_jump_top);
            jump_bottom = check_key(k_jump_bottom);
            mute_pressed = check_key(k_mute);
            accordion_open = check_key(k_accordion_open);
            accordion_close = check_key(k_accordion_close);

            // Global PTT Hotkeys (only when focused)
            if let Some(custom_key) = ptt_mode_egui_key {
                let pressed = match custom_key {
                    crate::ui::CustomKey::Egui(k) => i.key_pressed(k),
                    _ => false, 
                };
                if pressed {
                    let current_state = self.ptt_enabled.load(std::sync::atomic::Ordering::Relaxed);
                    let new_state = !current_state;
                    self.ptt_enabled.store(new_state, std::sync::atomic::Ordering::Relaxed);
                    let _ = self.tx_cmd.send(UICommand::SetGlobalMicMute { muted: new_state });
                }
            }

            if let Some(custom_key) = ptt_hold_egui_key {
                if self.ptt_enabled.load(std::sync::atomic::Ordering::Relaxed) {
                    let is_down = match custom_key {
                        crate::ui::CustomKey::Egui(k) => i.key_down(k),
                        crate::ui::CustomKey::Ctrl => i.modifiers.ctrl,
                        crate::ui::CustomKey::Alt => i.modifiers.alt,
                        crate::ui::CustomKey::Shift => i.modifiers.shift,
                    };

                    if is_down {
                        if !self.is_ptt_held {
                            self.is_ptt_held = true;
                            let _ = self.tx_cmd.send(UICommand::SetGlobalMicMute { muted: false });
                        }
                    } else {
                        if self.is_ptt_held {
                            self.is_ptt_held = false;
                            let _ = self.tx_cmd.send(UICommand::SetGlobalMicMute { muted: true });
                        }
                    }
                }
            }
        });

        let is_discord_selected = self.sessions.get(self.selected_index)
            .map_or(false, |s| s.name.to_lowercase().contains("discord"));

        if accordion_open && is_discord_selected {
            self.is_discord_accordion_open = true;
            self.selected_discord_user_index = 0;
        }

        if hide_requested || accordion_close {
            if self.is_discord_accordion_open {
                self.is_discord_accordion_open = false;
            } else {
                self.is_visible = false;
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
            }
        }

        let session_count = self.sessions.len();
        if session_count > 0 && self.is_visible {
            if nav_down {
                if self.is_discord_accordion_open && !self.discord_users.is_empty() {
                    self.selected_discord_user_index = (self.selected_discord_user_index + 1).min(self.discord_users.len() - 1);
                } else {
                    self.selected_index = (self.selected_index + 1).min(session_count - 1);
                }
            }
            if nav_up {
                if self.is_discord_accordion_open && !self.discord_users.is_empty() {
                    self.selected_discord_user_index = self.selected_discord_user_index.saturating_sub(1);
                } else {
                    self.selected_index = self.selected_index.saturating_sub(1);
                }
            }

            if mute_pressed {
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
                    let target_name = self.sessions[self.selected_index].name.clone();
                    
                    let is_muted = current_vol == 0.0;
                    let new_vol = if is_muted { 
                        self.saved_volumes.remove(&self.sessions[self.selected_index].pid).unwrap_or(100.0)
                    } else { 
                        0.0 
                    };

                    self.sessions[self.selected_index].volume = new_vol;
                    
                    for raw_session in &mut self.raw_sessions {
                        if raw_session.name == target_name {
                            if !is_muted {
                                self.saved_volumes.insert(raw_session.pid, raw_session.volume);
                            }
                            raw_session.volume = new_vol;
                            let _ = self.tx_cmd.send(UICommand::SetProcessVolume { pid: raw_session.pid, volume: new_vol });
                        }
                    }
                }
            }

            if jump_bottom { self.selected_index = session_count - 1; }
            if jump_top { self.selected_index = 0; }

            if vol_dec || vol_inc {
                let step = if fast_mod_active { 
                    self.config.settings.fast_step_percent 
                } else { 
                    self.config.settings.normal_step_percent 
                };
                
                let is_dec = vol_dec;

                if self.is_discord_accordion_open && !self.discord_users.is_empty() {
                    let user = &mut self.discord_users[self.selected_discord_user_index];
                    let mut new_vol = user.volume as f32;
                    if is_dec { new_vol = (new_vol - step).max(0.0); }
                    else { new_vol = (new_vol + step).min(200.0); }
                    user.volume = new_vol as u32;
                    
                    let _ = self.tx_cmd.send(UICommand::SetDiscordUserVolume {
                        user_id: user.id.clone(),
                        volume: user.volume,
                        mute: user.mute,
                    });
                } else {
                    let current_vol = self.sessions[self.selected_index].volume;
                    let new_vol = if is_dec { 
                        (current_vol - step).max(0.0) 
                    } else { 
                        (current_vol + step).min(100.0) 
                    };

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

        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                ui.add_space(4.0);
                for (index, session) in self.sessions.iter_mut().enumerate() {
                    let is_discord = session.name.to_lowercase().contains("discord");
                    let is_selected = index == self.selected_index && !self.is_discord_accordion_open;
                    
                    let background_color = if is_selected {
                        theme.bg_selection
                    } else {
                        egui::Color32::TRANSPARENT
                    };

                    let text_color = if is_selected {
                        theme.text_main
                    } else {
                        theme.text_dim
                    };

                    let row_response = egui::Frame::new()
                        .fill(background_color)
                        .corner_radius(theme.corner_radius)
                        .inner_margin(egui::Margin::symmetric(12, 8))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                // Selected Indicator Bar
                                if is_selected {
                                    let (rect, _) = ui.allocate_at_least(egui::vec2(3.0, 16.0), egui::Sense::hover());
                                    ui.painter().rect_filled(rect, 2.0, theme.primary_pink);
                                    ui.add_space(8.0);
                                } else {
                                    ui.add_space(11.0);
                                }

                                ui.label(egui::RichText::new(&session.name).color(text_color).strong());
                                
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if is_discord {
                                        let icon = if self.is_discord_accordion_open { "🔽" } else { "◀" };
                                        ui.label(egui::RichText::new(icon).color(text_color).size(12.0));
                                        ui.add_space(8.0);
                                    }

                                    // Always visible numeric volume
                                    ui.add_space(8.0);
                                    ui.label(egui::RichText::new(format!("{:>3}%", session.volume as u32))
                                        .color(if is_selected { theme.primary_pink } else { theme.text_dim })
                                        .monospace());
                                    
                                    let slider_response = ui.add(egui::Slider::new(&mut session.volume, 0.0..=100.0)
                                        .show_value(false));
                                    
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
                                ui.label(egui::RichText::new("Not in a Voice Channel").color(theme.text_dim).small());
                            } else {
                                for (i, user) in self.discord_users.iter_mut().enumerate() {
                                    let is_user_selected = i == self.selected_discord_user_index;
                                    let user_bg = if is_user_selected { theme.bg_selection.gamma_multiply(0.5) } else { egui::Color32::TRANSPARENT };
                                    let user_text = if is_user_selected { theme.text_main } else { theme.text_dim };
                                    
                                    ui.add_space(2.0);
                                    let user_row_response = egui::Frame::new()
                                        .fill(user_bg)
                                        .corner_radius(theme.corner_radius - 2.0)
                                        .inner_margin(egui::Margin::symmetric(10, 6))
                                        .show(ui, |ui| {
                                            ui.horizontal(|ui| {
                                                if is_user_selected {
                                                    let (rect, _) = ui.allocate_at_least(egui::vec2(2.0, 12.0), egui::Sense::hover());
                                                    ui.painter().rect_filled(rect, 1.0, theme.text_accent);
                                                    ui.add_space(6.0);
                                                } else {
                                                    ui.add_space(8.0);
                                                }

                                                let mute_icon = if user.mute { "🔇" } else { "🔊" };
                                                ui.label(egui::RichText::new(format!("{}  {}", mute_icon, user.username)).color(user_text));
                                                
                                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                    ui.add_space(8.0);
                                                    ui.label(egui::RichText::new(format!("{:>3}%", user.volume))
                                                        .color(if is_user_selected { theme.text_accent } else { theme.text_dim })
                                                        .monospace());

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
                    ui.add_space(4.0);
                }
            });
    }
}
