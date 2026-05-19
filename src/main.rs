#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
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
use tray_icon::menu::{Menu, MenuItem, MenuEvent};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

fn load_tray_icon() -> Icon {
    let icon_bytes = include_bytes!("../assets/icon.ico");
    let image = image::load_from_memory(icon_bytes).expect("Failed to parse icon image").into_rgba8();
    let (width, height) = image.dimensions();
    let rgba = image.into_raw();
    Icon::from_rgba(rgba, width, height).expect("Failed to create tray icon from image data")
}

fn parse_key(key_str: &str) -> rdev::Key {
    match key_str.to_uppercase().as_str() {
        "CONTROLLEFT" | "CTRL" | "LEFTCONTROL" => rdev::Key::ControlLeft,
        "CONTROLRIGHT" | "RIGHTCONTROL" => rdev::Key::ControlRight,
        "ALT" | "ALTLEFT" => rdev::Key::Alt,
        "ALTRIGHT" => rdev::Key::AltGr,
        "SHIFTLEFT" | "LEFTSHIFT" => rdev::Key::ShiftLeft,
        "SHIFTRIGHT" | "RIGHTSHIFT" => rdev::Key::ShiftRight,
        "BACKQUOTE" | "`" | "~" => rdev::Key::BackQuote,
        "SPACE" => rdev::Key::Space,
        "TAB" => rdev::Key::Tab,
        "CAPSLOCK" => rdev::Key::CapsLock,
        "A" => rdev::Key::KeyA,
        "B" => rdev::Key::KeyB,
        "C" => rdev::Key::KeyC,
        "D" => rdev::Key::KeyD,
        "E" => rdev::Key::KeyE,
        "F" => rdev::Key::KeyF,
        "G" => rdev::Key::KeyG,
        "H" => rdev::Key::KeyH,
        "I" => rdev::Key::KeyI,
        "J" => rdev::Key::KeyJ,
        "K" => rdev::Key::KeyK,
        "L" => rdev::Key::KeyL,
        "M" => rdev::Key::KeyM,
        "N" => rdev::Key::KeyN,
        "O" => rdev::Key::KeyO,
        "P" => rdev::Key::KeyP,
        "Q" => rdev::Key::KeyQ,
        "R" => rdev::Key::KeyR,
        "S" => rdev::Key::KeyS,
        "T" => rdev::Key::KeyT,
        "U" => rdev::Key::KeyU,
        "V" => rdev::Key::KeyV,
        "W" => rdev::Key::KeyW,
        "X" => rdev::Key::KeyX,
        "Y" => rdev::Key::KeyY,
        "Z" => rdev::Key::KeyZ,
        _ => rdev::Key::Unknown(0),
    }
}

#[tokio::main]
async fn main() {
    let mut app_config = config::AppConfig::load_or_create();
    let ptt_enabled_state = Arc::new(AtomicBool::new(false));

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
            .with_inner_size([550.0, 500.0]),
        ..Default::default()
    };

    let _ = eframe::run_native(
        "oto Overlay",
        native_options,
        Box::new(move |cc| {
            let ui_ctx = cc.egui_ctx.clone();

            let tray_menu = Menu::new();
            let quit_item = MenuItem::new("Quit", true, None);
            let _ = tray_menu.append(&quit_item);
            let quit_id = quit_item.id().clone();

            let tray_icon = TrayIconBuilder::new()
                .with_tooltip("oto")
                .with_icon(load_tray_icon())
                .with_menu(Box::new(tray_menu)) 
                .build()
                .unwrap();

            std::thread::spawn(move || {
                let menu_rx = MenuEvent::receiver();
                while let Ok(event) = menu_rx.recv() {
                    if event.id == quit_id {
                        println!("Quit selected from tray. Shutting down...");
                        std::process::exit(0);
                    }
                }
            });

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
                        if let Ok(local_user_id) = discord::authenticate_socket(&mut pipe, &token).await {
                            println!("Unified Persistent Discord Socket locked in! (User: {})", local_user_id);

                            let mut interval = tokio::time::interval(Duration::from_secs(2));
                            loop {
                                tokio::select! {
                                    _ = interval.tick() => {
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
            let overlay_key = parse_key(&app_config.hotkeys.toggle_overlay);
            let ptt_mode_key = parse_key(&app_config.hotkeys.ptt_mode_toggle);
            let ptt_hold_key = parse_key(&app_config.hotkeys.ptt_mic_hold);

            let tx_hotkey = tx.clone();
            let ctx_hotkey = ui_ctx.clone();
            let tx_hotkey_cmd = tx_cmd.clone();
            let ptt_state_for_rdev = ptt_enabled_state.clone(); 

            std::thread::spawn(move || {
                let mut is_ptt_held = false;
                let mut is_toggle_held = false;

                let callback = move |event: rdev::Event| {
                    match event.event_type {
                        rdev::EventType::KeyPress(key) => {
                            if key == overlay_key {
                                let _ = tx_hotkey.send(AppMessage::ToggleOverlay);
                                ctx_hotkey.request_repaint();
                            }

                            if key == ptt_mode_key {
                                if !is_toggle_held {
                                    is_toggle_held = true;
                                    let current_state = ptt_state_for_rdev.load(Ordering::Relaxed);
                                    let new_state = !current_state;
                                    ptt_state_for_rdev.store(new_state, Ordering::Relaxed);
                                    println!("[PTT] Mode Toggled: {}", if new_state { "ENABLED" } else { "DISABLED" });

                                    // Sync mic state immediately on mode toggle
                                    let _ = tx_hotkey_cmd.send(UICommand::SetGlobalMicMute { muted: new_state });
                                    ctx_hotkey.request_repaint();
                                }
                            }

                            if key == ptt_hold_key && ptt_state_for_rdev.load(Ordering::Relaxed) {
                                if !is_ptt_held {
                                    is_ptt_held = true;
                                    println!("[PTT] Key Pressed - Unmuting Mic");
                                    let _ = tx_hotkey_cmd.send(UICommand::SetGlobalMicMute { muted: false }); 
                                }
                            }
                        }
                        rdev::EventType::KeyRelease(key) => {
                            if key == ptt_mode_key {
                                is_toggle_held = false;
                            }

                            if key == ptt_hold_key {
                                if is_ptt_held {
                                    is_ptt_held = false;
                                    println!("[PTT] Key Released - Muting Mic");
                                    // Only mute if PTT mode is still enabled
                                    if ptt_state_for_rdev.load(Ordering::Relaxed) {
                                        let _ = tx_hotkey_cmd.send(UICommand::SetGlobalMicMute { muted: true }); 
                                    }
                                }
                            }
                        }
                        _ => {}
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

                let mut global_mic = audio::wasapi::PersistentMic::new().expect("Failed to bind to Windows Mic");
                let mut mic_refresh_interval = tokio::time::interval(Duration::from_secs(5));

                if let Ok(actual_sessions) = audio::wasapi::get_active_sessions() {
                    let _ = tx_engine.send(app::AppMessage::UpdateSessions(actual_sessions));
                }

                loop {
                    tokio::select! {
                        _ = mic_refresh_interval.tick() => {
                            if let Err(e) = global_mic.refresh() {
                                println!("Failed to refresh mic pointer: {}", e);
                            }
                        }
                        cmd_opt = rx_cmd.recv() => {
                            if let Some(cmd) = cmd_opt {
                                match cmd {
                                    UICommand::SetProcessVolume { pid, volume } => {
                                        let _ = audio::wasapi::set_process_volume(pid, volume);
                                    }
                                    UICommand::SetDiscordUserVolume { user_id, volume, mute } => {
                                        let _ = tx_discord_writer.send((user_id, volume, mute));
                                    }
                                    UICommand::SetGlobalMicMute { muted } => {
                                        if let Err(e) = global_mic.set_mute(muted) {
                                            println!("Failed to toggle global mic: {}", e);
                                        }
                                    }
                                }
                            } else {
                                break;
                            }
                        }
                    }
                }
            });

            Ok(Box::new(ui::MixerApp::new(rx, tx_cmd, tray_icon, app_config, ptt_enabled_state.clone())))
        }),
    );
}