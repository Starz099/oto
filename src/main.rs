#![windows_subsystem = "windows"]
mod app;
mod ui;
mod audio;
mod config;
mod discord;

use tokio::sync::mpsc;
use crate::app::{AppMessage, UICommand};
use std::time::Duration;
use eframe::egui;
use tray_icon::{TrayIconBuilder, Icon, TrayIconEvent, MouseButton, MouseButtonState};

fn load_tray_icon() -> Icon {
    let icon_bytes = include_bytes!("../assets/icon.png");
    let image = image::load_from_memory(icon_bytes).expect("Failed to parse icon image").into_rgba8();
    let (width, height) = image.dimensions();
    let rgba = image.into_raw();
    Icon::from_rgba(rgba, width, height).expect("Failed to create tray icon from image data")
}

#[tokio::main]
async fn main() {
    let mut app_config = config::AppConfig::load_or_create();
    
    if let Some(_token) = &app_config.discord_access_token {
        println!("Found saved Discord token. Skipping auth popup.");
    } else {
        println!("No token found in config. Starting Discord auth flow...");
        match discord::get_access_token().await {
            Ok(new_token) => {
                println!("Successfully retrieved new Discord token.");
                app_config.discord_access_token = Some(new_token);
                app_config.save(); 
            }
            Err(e) => println!("OAUTH FAILED: {}", e),
        }
    }

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

            let tray_icon = TrayIconBuilder::new()
                .with_tooltip("Raw Mixer")
                .with_icon(load_tray_icon())
                .build()
                .unwrap();

            // WASAPI Background Poller
            let tx_poller = tx.clone();
            let ctx_poller = ui_ctx.clone();
            tokio::spawn(async move {
                loop {
                    if let Ok(actual_sessions) = audio::wasapi::get_active_sessions() {
                        let _ = tx_poller.send(AppMessage::UpdateSessions(actual_sessions));
                        ctx_poller.request_repaint();
                    }
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            });

            // UNIFIED Persistent Discord Engine
            let discord_token_engine = app_config.discord_access_token.clone();
            let (tx_discord_writer, mut rx_discord_writer) = tokio::sync::mpsc::unbounded_channel::<(String, u32, bool)>();
            let tx_discord_ui = tx.clone();
            let ctx_discord_ui = ui_ctx.clone();

            if let Some(token) = discord_token_engine {
                tokio::spawn(async move {
                    if let Ok(mut pipe) = discord::connect_to_discord().await {
                        // Catch the returned ID here!
                        if let Ok(local_user_id) = discord::authenticate_socket(&mut pipe, &token).await {
                            println!("Unified Persistent Discord Socket locked in! (User: {})", local_user_id);
                            
                            let mut interval = tokio::time::interval(Duration::from_secs(2));

                            loop {
                                tokio::select! {
                                    _ = interval.tick() => {
                                        // Pass the ID into the fetcher so it knows who to hide
                                        if let Ok(users) = discord::get_current_vc_users_persistent(&mut pipe, &local_user_id).await {
                                            let _ = tx_discord_ui.send(crate::app::AppMessage::UpdateDiscordUsers(users));
                                            ctx_discord_ui.request_repaint();
                                        }
                                    }
                                    cmd = rx_discord_writer.recv() => {
                                        if let Some((user_id, volume, mute)) = cmd {
                                            let _ = discord::set_user_voice_settings_persistent(&mut pipe, &user_id, volume, mute).await;
                                        }
                                    }
                                }
                            }
                        }
                    }
                });
            }

            // Hotkey Listener
            let tx_hotkey = tx.clone();
            let ctx_hotkey = ui_ctx.clone();
            std::thread::spawn(move || {
                let callback = move |event: rdev::Event| {
                    if let rdev::EventType::KeyPress(key) = event.event_type {
                        if key == rdev::Key::BackQuote {
                            let _ = tx_hotkey.send(AppMessage::ToggleOverlay);
                            ctx_hotkey.request_repaint();
                        }
                    }
                };
                let _ = rdev::listen(callback);
            });

            // Tray Icon Listener
            let tx_tray = tx.clone();
            let ctx_tray = ui_ctx.clone();
            std::thread::spawn(move || {
                let tray_rx = TrayIconEvent::receiver();
                while let Ok(event) = tray_rx.recv() {
                    if let TrayIconEvent::Click { button, button_state, .. } = event {
                        if button == MouseButton::Left && button_state == MouseButtonState::Up {
                            let _ = tx_tray.send(AppMessage::ToggleOverlay);
                            ctx_tray.request_repaint();
                        }
                    }
                }
            });

            // Main Backend Engine Command Router
            let tx_engine = tx.clone();
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(500)).await;

                if let Ok(actual_sessions) = audio::wasapi::get_active_sessions() {
                    let _ = tx_engine.send(app::AppMessage::UpdateSessions(actual_sessions));
                }

                while let Some(cmd) = rx_cmd.recv().await {
                    match cmd {
                        UICommand::SetProcessVolume { pid, volume } => {
                            let _ = audio::wasapi::set_process_volume(pid, volume);
                        }
                        UICommand::SetDiscordUserVolume { user_id, volume, mute } => {
                            let _ = tx_discord_writer.send((user_id, volume, mute));
                        }
                    }
                }
            });

            Ok(Box::new(ui::MixerApp::new(rx, tx_cmd, tray_icon, app_config)))
        }),
    );
}