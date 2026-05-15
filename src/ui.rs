use eframe::egui;

pub fn run_overlay() {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_decorations(false)
            .with_transparent(true)
            .with_always_on_top()
            .with_inner_size([450.0, 350.0]),
        ..Default::default()
    };

    let _ = eframe::run_native(
        "Raw Mixer Overlay", 
        native_options, 
        Box::new(|cc| Ok(Box::new(MixerApp::new(cc))))
    );
}

pub struct MixerApp {
    initialized: bool,
    starz_vol: f32,
}

impl Default for MixerApp {
    fn default() -> Self {
        Self {
            initialized: false,
            starz_vol: 56.0,
        }
    }
}

impl MixerApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }
}

impl eframe::App for MixerApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0] 
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        
        if !self.initialized {
            let mut visuals = egui::Visuals::dark();
            visuals.panel_fill = egui::Color32::TRANSPARENT;
            visuals.window_fill = egui::Color32::TRANSPARENT;
            ui.ctx().set_visuals(visuals);
            
            // Try to center the viewport (Works on most Windows machines)
            // If the monitor size is known, it moves the widget to the center
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
            .fill(egui::Color32::from_rgba_premultiplied(30, 30, 30, 200)) 
            .corner_radius(15.0)
            .inner_margin(15.0);

        egui::CentralPanel::default().frame(panel_frame).show_inside(ui, |ui| {
            let app_rect = ui.max_rect();
            let title_bar_response = ui.interact(app_rect, ui.id().with("window_drag"), egui::Sense::drag());
            if title_bar_response.dragged() {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
            }

            ui.heading(egui::RichText::new("Raw Mixer Control")
            .size(24.0)
            .color(egui::Color32::WHITE));
            ui.separator();
            
            ui.add_space(10.0);
            ui.label("Global Push-To-Talk: [ACTIVE]");
            
            ui.add_space(20.0);
            ui.heading("Active VC: VC Name");
            
            ui.horizontal(|ui| {
                ui.label("Starz");
                let mut vol = self.starz_vol;
                ui.add(egui::Slider::new(&mut vol, 0.0..=200.0).text("%"));
                self.starz_vol = vol;
            });
        });
    }
}