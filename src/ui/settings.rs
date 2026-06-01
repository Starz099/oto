use eframe::egui;
use crate::ui::MixerApp;
use crate::ui::theme;
use std::sync::Arc;

impl MixerApp {
    pub(crate) fn show_settings_ui(&mut self, ui: &mut egui::Ui) {
        let theme = theme::Theme::pastel_pink();
        ui.add_space(8.0);
        
        let mut config = (*self.config).clone();
        let mut config_changed = false;

        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .max_height(ui.available_height() - 80.0)
            .show(ui, |ui| {
                ui.label(egui::RichText::new("Keybindings").color(theme.primary_pink).strong());
                ui.add_space(theme.item_spacing / 2.0);

                self.keybind_row_styled(ui, "Toggle Overlay", "toggle_overlay", &theme);
                self.keybind_row_styled(ui, "PTT Mode Toggle", "ptt_mode_toggle", &theme);
                self.keybind_row_styled(ui, "PTT Hold Key", "ptt_mic_hold", &theme);
                
                ui.add_space(12.0);
                ui.label(egui::RichText::new("Navigation").color(theme.text_accent).small());
                ui.add_space(4.0);
                
                self.keybind_row_styled(ui, "Navigate Up", "nav_up", &theme);
                self.keybind_row_styled(ui, "Navigate Down", "nav_down", &theme);
                self.keybind_row_styled(ui, "Jump to Top", "jump_top", &theme);
                self.keybind_row_styled(ui, "Jump to Bottom", "jump_bottom", &theme);
                self.keybind_row_styled(ui, "Open Accordion", "accordion_open", &theme);
                self.keybind_row_styled(ui, "Close Accordion", "accordion_close", &theme);
                
                ui.add_space(12.0);
                ui.label(egui::RichText::new("Volume Control").color(theme.text_accent).small());
                ui.add_space(4.0);
                
                self.keybind_row_styled(ui, "Volume Increase", "vol_increase", &theme);
                self.keybind_row_styled(ui, "Volume Decrease", "vol_decrease", &theme);
                self.keybind_row_styled(ui, "Fast Step Modifier", "fast_modifier", &theme);
                self.keybind_row_styled(ui, "Mute", "mute", &theme);

                ui.add_space(24.0);
                ui.label(egui::RichText::new("General Settings").color(theme.primary_pink).strong());
                ui.add_space(theme.item_spacing / 2.0);

                if Self::slider_row_styled(ui, "Normal Step (%)", &mut config.settings.normal_step_percent, 1.0..=20.0, &theme) { config_changed = true; }
                if Self::slider_row_styled(ui, "Fast Step (%)", &mut config.settings.fast_step_percent, 5.0..=50.0, &theme) { config_changed = true; }

                ui.add_space(24.0);
                ui.label(egui::RichText::new("Discord API (Bring Your Own Credentials)").color(theme.primary_pink).strong());
                ui.add_space(theme.item_spacing / 2.0);
                ui.label(egui::RichText::new("You need to create a Discord Application at discord.com/developers").color(theme.text_accent).small());
                
                let mut client_id = config.discord_client_id.clone().unwrap_or_default();
                if self.text_input_row_styled(ui, "Client ID", &mut client_id, &theme) {
                    config.discord_client_id = if client_id.is_empty() { None } else { Some(client_id) };
                    config_changed = true;
                    self.needs_restart = true;
                }

                let mut client_secret = config.discord_client_secret.clone().unwrap_or_default();
                if self.text_input_row_styled(ui, "Client Secret", &mut client_secret, &theme) {
                    config.discord_client_secret = if client_secret.is_empty() { None } else { Some(client_secret) };
                    config_changed = true;
                    self.needs_restart = true;
                }

                if config.discord_access_token.is_some() {
                    ui.add_space(8.0);
                    if ui.button("Clear Token (Re-authenticate)").clicked() {
                        config.discord_access_token = None;
                        config_changed = true;
                        self.needs_restart = true;
                    }
                }

                ui.add_space(16.0);
            });

        ui.add_space(12.0);
        let button_label = if self.needs_restart {
            "Apply and Restart App"
        } else {
            "Save Changes"
        };

        let (button_fill, button_text_color, button_border) = if self.needs_restart {
            (theme.primary_pink.gamma_multiply(0.22), theme.bg_dark, theme.primary_pink)
        } else {
            (theme.bg_selection, theme.text_main, theme.primary_pink.gamma_multiply(0.5))
        };

        let save_btn = ui.add_sized([ui.available_width(), 44.0], 
            egui::Button::new(egui::RichText::new(button_label).strong().size(16.0).color(button_text_color))
                .fill(button_fill)
                .stroke(egui::Stroke::new(1.0, button_border))
        );

        if save_btn.clicked() {
            config.save();
            self.config = Arc::new(config.clone());
            if self.needs_restart {
                self.restart_app();
            }
        }
        ui.add_space(12.0);

        // Key Grabber Logic
        if let Some(field_name) = self.recording_keybind.clone() {
            let mut captured_key = None;
            
            ui.ctx().input(|i| {
                for event in &i.events {
                    if let egui::Event::Key { key, pressed: true, .. } = event {
                        if *key != egui::Key::Escape {
                            captured_key = Some(crate::ui::CustomKey::Egui(*key));
                        } else {
                            self.recording_keybind = None;
                        }
                    }
                }

                if captured_key.is_none() {
                    if i.modifiers.ctrl { captured_key = Some(crate::ui::CustomKey::Ctrl); }
                    else if i.modifiers.shift { captured_key = Some(crate::ui::CustomKey::Shift); }
                    else if i.modifiers.alt { captured_key = Some(crate::ui::CustomKey::Alt); }
                }
            });

            if let Some(key) = captured_key {
                let key_str = self.custom_key_to_str(key);
                match field_name.as_str() {
                    "toggle_overlay" => config.hotkeys.toggle_overlay = key_str,
                    "ptt_mode_toggle" => config.hotkeys.ptt_mode_toggle = key_str,
                    "ptt_mic_hold" => config.hotkeys.ptt_mic_hold = key_str,
                    "nav_up" => config.hotkeys.nav_up = key_str,
                    "nav_down" => config.hotkeys.nav_down = key_str,
                    "vol_increase" => config.hotkeys.vol_increase = key_str,
                    "vol_decrease" => config.hotkeys.vol_decrease = key_str,
                    "fast_modifier" => config.hotkeys.fast_modifier = key_str,
                    "jump_top" => config.hotkeys.jump_top = key_str,
                    "jump_bottom" => config.hotkeys.jump_bottom = key_str,
                    "accordion_open" => config.hotkeys.accordion_open = key_str,
                    "accordion_close" => config.hotkeys.accordion_close = key_str,
                    "mute" => config.hotkeys.mute = key_str,
                    _ => {}
                }

                if config.hotkeys.toggle_overlay != self.original_hotkeys.toggle_overlay ||
                   config.hotkeys.ptt_mode_toggle != self.original_hotkeys.ptt_mode_toggle ||
                   config.hotkeys.ptt_mic_hold != self.original_hotkeys.ptt_mic_hold {
                    self.needs_restart = true;
                }

                self.recording_keybind = None;
                config_changed = true;
            }
        }

        if config_changed {
            self.config = Arc::new(config);
        }
    }

    fn restart_app(&self) {
        if let Ok(current_exe) = std::env::current_exe() {
            let _ = std::process::Command::new(current_exe).spawn();
            std::process::exit(0);
        }
    }

    fn text_input_row_styled(&self, ui: &mut egui::Ui, label: &str, value: &mut String, theme: &theme::Theme) -> bool {
        ui.add_space(2.0);
        let mut changed = false;
        egui::Frame::new()
            .fill(theme.bg_panel.gamma_multiply(0.2))
            .inner_margin(egui::Margin::symmetric(12, 8))
            .corner_radius(theme.corner_radius)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(label).color(theme.text_main));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let response = ui.add(egui::TextEdit::singleline(value)
                            .min_size(egui::vec2(250.0, 24.0))
                            .margin(egui::Margin::symmetric(8, 4))
                        );
                        if response.changed() {
                            changed = true;
                        }
                    });
                });
            });
        changed
    }

    fn keybind_row_styled(&mut self, ui: &mut egui::Ui, label: &str, field_name: &str, theme: &theme::Theme) {
        ui.add_space(2.0);
        egui::Frame::new()
            .fill(theme.bg_panel.gamma_multiply(0.2))
            .inner_margin(egui::Margin::symmetric(12, 8))
            .corner_radius(theme.corner_radius)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(label).color(theme.text_main));
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let current_val = match field_name {
                            "toggle_overlay" => &self.config.hotkeys.toggle_overlay,
                            "ptt_mode_toggle" => &self.config.hotkeys.ptt_mode_toggle,
                            "ptt_mic_hold" => &self.config.hotkeys.ptt_mic_hold,
                            "nav_up" => &self.config.hotkeys.nav_up,
                            "nav_down" => &self.config.hotkeys.nav_down,
                            "vol_increase" => &self.config.hotkeys.vol_increase,
                            "vol_decrease" => &self.config.hotkeys.vol_decrease,
                            "fast_modifier" => &self.config.hotkeys.fast_modifier,
                            "jump_top" => &self.config.hotkeys.jump_top,
                            "jump_bottom" => &self.config.hotkeys.jump_bottom,
                            "accordion_open" => &self.config.hotkeys.accordion_open,
                            "accordion_close" => &self.config.hotkeys.accordion_close,
                            "mute" => &self.config.hotkeys.mute,
                            _ => "",
                        };

                        let is_recording = self.recording_keybind.as_deref() == Some(field_name);
                        let button_text = if is_recording { "..." } else { current_val };

                        let btn = ui.add(egui::Button::new(egui::RichText::new(button_text).monospace())
                            .min_size(egui::vec2(100.0, 28.0))
                            .fill(if is_recording { theme.primary_pink } else { theme.bg_selection }));
                        
                        if btn.clicked() {
                            self.recording_keybind = Some(field_name.to_string());
                        }
                    });
                });
            });
    }

    fn slider_row_styled(ui: &mut egui::Ui, label: &str, value: &mut f32, range: std::ops::RangeInclusive<f32>, theme: &theme::Theme) -> bool {
        ui.add_space(2.0);
        let mut changed = false;
        egui::Frame::new()
            .fill(theme.bg_panel.gamma_multiply(0.2))
            .inner_margin(egui::Margin::symmetric(12, 8))
            .corner_radius(theme.corner_radius)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(label).color(theme.text_main));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add(egui::Slider::new(value, range).show_value(true)).changed() {
                            changed = true;
                        }
                    });
                });
            });
        changed
    }

    fn custom_key_to_str(&self, key: crate::ui::CustomKey) -> String {
        match key {
            crate::ui::CustomKey::Ctrl => "LeftControl".to_string(),
            crate::ui::CustomKey::Shift => "LeftShift".to_string(),
            crate::ui::CustomKey::Alt => "Alt".to_string(),
            crate::ui::CustomKey::Egui(k) => self.egui_key_to_str(k),
        }
    }

    fn egui_key_to_str(&self, key: egui::Key) -> String {
        match key {
            egui::Key::A => "A".to_string(),
            egui::Key::B => "B".to_string(),
            egui::Key::C => "C".to_string(),
            egui::Key::D => "D".to_string(),
            egui::Key::E => "E".to_string(),
            egui::Key::F => "F".to_string(),
            egui::Key::G => "G".to_string(),
            egui::Key::H => "H".to_string(),
            egui::Key::I => "I".to_string(),
            egui::Key::J => "J".to_string(),
            egui::Key::K => "K".to_string(),
            egui::Key::L => "L".to_string(),
            egui::Key::M => "M".to_string(),
            egui::Key::N => "N".to_string(),
            egui::Key::O => "O".to_string(),
            egui::Key::P => "P".to_string(),
            egui::Key::Q => "Q".to_string(),
            egui::Key::R => "R".to_string(),
            egui::Key::S => "S".to_string(),
            egui::Key::T => "T".to_string(),
            egui::Key::U => "U".to_string(),
            egui::Key::V => "V".to_string(),
            egui::Key::W => "W".to_string(),
            egui::Key::X => "X".to_string(),
            egui::Key::Y => "Y".to_string(),
            egui::Key::Z => "Z".to_string(),
            egui::Key::Num0 => "0".to_string(),
            egui::Key::Num1 => "1".to_string(),
            egui::Key::Num2 => "2".to_string(),
            egui::Key::Num3 => "3".to_string(),
            egui::Key::Num4 => "4".to_string(),
            egui::Key::Num5 => "5".to_string(),
            egui::Key::Num6 => "6".to_string(),
            egui::Key::Num7 => "7".to_string(),
            egui::Key::Num8 => "8".to_string(),
            egui::Key::Num9 => "9".to_string(),
            egui::Key::F1 => "F1".to_string(),
            egui::Key::F2 => "F2".to_string(),
            egui::Key::F3 => "F3".to_string(),
            egui::Key::F4 => "F4".to_string(),
            egui::Key::F5 => "F5".to_string(),
            egui::Key::F6 => "F6".to_string(),
            egui::Key::F7 => "F7".to_string(),
            egui::Key::F8 => "F8".to_string(),
            egui::Key::F9 => "F9".to_string(),
            egui::Key::F10 => "F10".to_string(),
            egui::Key::F11 => "F11".to_string(),
            egui::Key::F12 => "F12".to_string(),
            egui::Key::Backtick => "BackQuote".to_string(),
            egui::Key::Space => "Space".to_string(),
            egui::Key::Tab => "Tab".to_string(),
            egui::Key::Enter => "Enter".to_string(),
            egui::Key::Escape => "Escape".to_string(),
            _ => format!("{:?}", key),
        }
    }
}
