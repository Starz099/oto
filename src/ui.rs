use eframe::egui;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use crate::app::{AppMessage, AudioProcess, UICommand};

pub struct MixerApp {
    initialized: bool,
    is_visible: bool,
    sessions: Vec<AudioProcess>,
    rx: UnboundedReceiver<AppMessage>,
    tx_cmd: UnboundedSender<UICommand>,
}

impl MixerApp {
    pub fn new(rx: UnboundedReceiver<AppMessage>, tx_cmd: UnboundedSender<UICommand>) -> Self {
        Self {
            initialized: false,
            is_visible: true,
            sessions: Vec::new(),
            rx,
            tx_cmd,
        }
    }
}

impl eframe::App for MixerApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0] 
    }
    //TODO: Move it to a separate file for better code organization
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                AppMessage::UpdateSessions(sessions) => {
                    self.sessions = sessions;
                }
                AppMessage::ToggleOverlay => {
                    self.is_visible = !self.is_visible;
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Visible(self.is_visible));
                    println!("Overlay visibility toggled: {}", self.is_visible);
                    if self.is_visible {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Focus);
                        if let Some(monitor_size) = ui.ctx().input(|i| i.viewport().monitor_size) {
                            let center_pos = egui::pos2(
                                (monitor_size.x - 450.0) / 2.0,
                                (monitor_size.y - 350.0) / 2.0,
                            );
                            ui.ctx().send_viewport_cmd(egui::ViewportCommand::OuterPosition(center_pos));
                        }
                    }
                }
            }
        }

        let mut hide_requested = false;
        ui.input(|i| {
            // Standard UX: Escape key should always minimize overlays
            if i.key_pressed(egui::Key::Escape) {
                hide_requested = true;
            }
            
            // Check if Tilde/Backtick was pressed
            for event in &i.events {
                if let egui::Event::Text(text) = event {
                    if text.contains('`') || text.contains('~') {
                        hide_requested = true;
                    }
                }
            }
        });

        if hide_requested {
            self.is_visible = false;
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Visible(false));
        }


        if !self.initialized {
            let mut visuals = egui::Visuals::dark();
            visuals.panel_fill = egui::Color32::TRANSPARENT;
            visuals.window_fill = egui::Color32::TRANSPARENT;
            let no_shadow = egui::epaint::Shadow {
                offset: [0, 0],
                blur: 0,
                spread: 0,
                color: egui::Color32::TRANSPARENT,
            };
            visuals.window_shadow = no_shadow;
            visuals.popup_shadow = no_shadow;

            ui.ctx().set_visuals(visuals);
            
            if let Some(monitor_size) = ui.ctx().input(|i| i.viewport().monitor_size) {
                let center_pos = egui::pos2(
                    (monitor_size.x - 450.0) / 2.0,
                    (monitor_size.y - 350.0) / 2.0,
                );
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::OuterPosition(center_pos));
            }
            
            self.initialized = true;
        }

        let panel_frame = egui::Frame::new()
            .fill(egui::Color32::from_rgba_unmultiplied(80, 70, 90, 240)) 
            .corner_radius(15.0)
            .inner_margin(15.0)
            .shadow(egui::epaint::Shadow { 
                offset: [0, 0],
                blur: 0, 
                spread: 0, 
                color: egui::Color32::TRANSPARENT 
            });

        egui::CentralPanel::default().frame(panel_frame).show_inside(ui, |ui| {
            let app_rect = ui.max_rect();
            let title_bar_response = ui.interact(app_rect, ui.id().with("window_drag"), egui::Sense::drag());
            if title_bar_response.dragged() {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
            }

            ui.heading(egui::RichText::new("Raw Mixer")
            .size(24.0)
            .color(egui::Color32::WHITE));
            ui.separator();
                                    
            ui.add_space(40.0);

            for session in &mut self.sessions {
                ui.horizontal(|ui| {
                    ui.label(format!("{} (PID: {})", session.name, session.pid));                    
                    let slider_response = ui.add(egui::Slider::new(&mut session.volume, 0.0..=100.0).text("%"));
                    
                    if slider_response.changed() {
                        let _ = self.tx_cmd.send(UICommand::SetProcessVolume { 
                            pid: session.pid, 
                            volume: session.volume 
                        });
                    }
                });
                ui.add_space(10.0);
            }
        });
    }
}