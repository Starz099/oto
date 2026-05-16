mod app;
mod ui;
mod audio;

use tokio::sync::mpsc;
use crate::app::{AppMessage, UICommand};
use std::time::Duration;
use eframe::egui;

#[tokio::main]
async fn main() {
    let (tx, rx) = mpsc::unbounded_channel::<AppMessage>();
    let (tx_cmd, mut rx_cmd) = mpsc::unbounded_channel::<UICommand>();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_decorations(false)
            .with_transparent(true)
            .with_always_on_top()
            .with_taskbar(false)
            .with_inner_size([450.0, 350.0]),
        ..Default::default()
    };

    let _ = eframe::run_native(
        "Raw Mixer Overlay",
        native_options,
        Box::new(move |cc| {
            let ui_ctx = cc.egui_ctx.clone();

            let tx_clone = tx.clone();
            let ctx_poller = ui_ctx.clone();
            
            tokio::spawn(async move {
                println!("Background Poller started. Fetching sessions every 1 second...");
                loop {
                    if let Ok(actual_sessions) = audio::wasapi::get_active_sessions() {
                        let _ = tx_clone.send(AppMessage::UpdateSessions(actual_sessions));
                        ctx_poller.request_repaint();
                    }
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            });

            let tx_clone_2 = tx.clone();
            let ctx_hotkey = ui_ctx.clone();
            
            std::thread::spawn(move || {
                println!("Hotkey Listener started. Press ` (Tilde) to toggle overlay...");
                let callback = move |event: rdev::Event| {
                    if let rdev::EventType::KeyPress(key) = event.event_type {
                        if key == rdev::Key::BackQuote {
                            println!("Toggle hotkey pressed. Toggling overlay visibility...");
                            let _ = tx_clone_2.send(AppMessage::ToggleOverlay);
                            println!("Overlay toggle command sent to UI.");
                            ctx_hotkey.request_repaint();
                        }
                    }
                };

                if let Err(error) = rdev::listen(callback) {
                    println!("Error listening to keyboard: {:?}", error);
                }
            });

            let tx_clone_3 = tx.clone();
            
            tokio::spawn(async move {
                println!("Backend Engine started. Listening for UI commands...");
                tokio::time::sleep(Duration::from_millis(500)).await;

                if let Ok(actual_sessions) = audio::wasapi::get_active_sessions() {
                    let _ = tx_clone_3.send(app::AppMessage::UpdateSessions(actual_sessions));
                }

                while let Some(cmd) = rx_cmd.recv().await {
                    match cmd {
                        UICommand::SetProcessVolume { pid, volume } => {
                            match audio::wasapi::set_process_volume(pid, volume) {
                                Ok(_) => println!("Successfully updated PID {} to {:.1}%", pid, volume),
                                Err(e) => println!("Error updating PID {}: {}", pid, e),
                            }
                        }
                    }
                }
            });

            Ok(Box::new(ui::MixerApp::new(rx, tx_cmd)))
        }),
    );
}