use eframe::egui;
use crate::ui::MixerApp;

impl MixerApp {
    pub(crate) fn show_settings_ui(&mut self, ui: &mut egui::Ui) {
        ui.add_space(16.0);
        ui.heading(egui::RichText::new("Settings").color(egui::Color32::from_rgb(200, 200, 200)));
        ui.add_space(8.0);

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.heading(egui::RichText::new("Keybindings").size(18.0).color(egui::Color32::from_rgb(180, 180, 180)));
            ui.add_space(8.0);

            egui::Frame::new()
                .fill(egui::Color32::from_rgb(25, 25, 25))
                .inner_margin(12.0)
                .corner_radius(4.0)
                .show(ui, |ui| {
                    egui::Grid::new("hotkeys_grid")
                        .num_columns(2)
                        .spacing([40.0, 12.0])
                        .show(ui, |ui| {
                            self.keybind_row(ui, "Toggle Overlay", "toggle_overlay");
                            self.keybind_row(ui, "PTT Mode Toggle", "ptt_mode_toggle");
                            self.keybind_row(ui, "PTT Hold Key", "ptt_mic_hold");
                            ui.end_row();
                            ui.label(egui::RichText::new("Navigation").strong());
                            ui.end_row();
                            self.keybind_row(ui, "Navigate Up", "nav_up");
                            self.keybind_row(ui, "Navigate Down", "nav_down");
                            self.keybind_row(ui, "Jump to Top", "jump_top");
                            self.keybind_row(ui, "Jump to Bottom", "jump_bottom");
                            self.keybind_row(ui, "Open Accordion", "accordion_open");
                            self.keybind_row(ui, "Close Accordion", "accordion_close");
                            ui.end_row();
                            ui.label(egui::RichText::new("Volume").strong());
                            ui.end_row();
                            self.keybind_row(ui, "Volume Increase", "vol_increase");
                            self.keybind_row(ui, "Volume Decrease", "vol_decrease");
                            self.keybind_row(ui, "Fast Step Modifier", "fast_modifier");
                            self.keybind_row(ui, "Mute", "mute");
                        });
                });

            ui.add_space(24.0);
            ui.heading(egui::RichText::new("General").size(18.0).color(egui::Color32::from_rgb(180, 180, 180)));
            ui.add_space(8.0);

            egui::Frame::new()
                .fill(egui::Color32::from_rgb(25, 25, 25))
                .inner_margin(12.0)
                .corner_radius(4.0)
                .show(ui, |ui| {
                    egui::Grid::new("settings_grid")
                        .num_columns(2)
                        .spacing([40.0, 12.0])
                        .show(ui, |ui| {
                            ui.label("Normal Step (%):");
                            ui.add(egui::Slider::new(&mut self.config.settings.normal_step_percent, 1.0..=20.0));
                            ui.end_row();

                            ui.label("Fast Step (%):");
                            ui.add(egui::Slider::new(&mut self.config.settings.fast_step_percent, 5.0..=50.0));
                            ui.end_row();
                        });
                });

            ui.add_space(24.0);
            let button_label = if self.needs_restart {
                "💾 Save and Restart"
            } else {
                "💾 Save All Settings"
            };

            if ui.button(egui::RichText::new(button_label).size(16.0)).clicked() {
                self.config.save();
                if self.needs_restart {
                    self.restart_app();
                }
            }
        });

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
                    "toggle_overlay" => self.config.hotkeys.toggle_overlay = key_str,
                    "ptt_mode_toggle" => self.config.hotkeys.ptt_mode_toggle = key_str,
                    "ptt_mic_hold" => self.config.hotkeys.ptt_mic_hold = key_str,
                    "nav_up" => self.config.hotkeys.nav_up = key_str,
                    "nav_down" => self.config.hotkeys.nav_down = key_str,
                    "vol_increase" => self.config.hotkeys.vol_increase = key_str,
                    "vol_decrease" => self.config.hotkeys.vol_decrease = key_str,
                    "fast_modifier" => self.config.hotkeys.fast_modifier = key_str,
                    "jump_top" => self.config.hotkeys.jump_top = key_str,
                    "jump_bottom" => self.config.hotkeys.jump_bottom = key_str,
                    "accordion_open" => self.config.hotkeys.accordion_open = key_str,
                    "accordion_close" => self.config.hotkeys.accordion_close = key_str,
                    "mute" => self.config.hotkeys.mute = key_str,
                    _ => {}
                }

                // Check if global listener keys were changed
                if self.config.hotkeys.toggle_overlay != self.original_hotkeys.toggle_overlay ||
                   self.config.hotkeys.ptt_mode_toggle != self.original_hotkeys.ptt_mode_toggle ||
                   self.config.hotkeys.ptt_mic_hold != self.original_hotkeys.ptt_mic_hold {
                    self.needs_restart = true;
                }

                self.recording_keybind = None;
            }
        }
    }

    fn restart_app(&self) {
        if let Ok(current_exe) = std::env::current_exe() {
            let _ = std::process::Command::new(current_exe).spawn();
            std::process::exit(0);
        }
    }

    fn keybind_row(&mut self, ui: &mut egui::Ui, label: &str, field_name: &str) {
        ui.label(label);
        
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
        let button_text = if is_recording {
            "Press a key...".to_string()
        } else {
            current_val.to_string()
        };

        if ui.button(egui::RichText::new(button_text).color(if is_recording { egui::Color32::LIGHT_BLUE } else { egui::Color32::WHITE })).clicked() {
            self.recording_keybind = Some(field_name.to_string());
        }
        ui.end_row();
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
