#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod app;
mod ui;
mod audio;
mod config;
mod discord;
mod core;
mod input;

use tokio::sync::mpsc;
use crate::app::{AppMessage, UICommand};
use std::time::Duration;
use eframe::egui;
use tray_icon::{TrayIconBuilder, Icon, TrayIconEvent, MouseButton, MouseButtonState};
use tray_icon::menu::{Menu, MenuItem, MenuEvent};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

fn load_tray_icon() -> Icon {
    let icon_bytes = include_bytes!("../assets/icon.ico");
    let image = image::load_from_memory(icon_bytes).expect("Failed to parse icon image").into_rgba8();
    let (width, height) = image.dimensions();
    let rgba = image.into_raw();
    Icon::from_rgba(rgba, width, height).expect("Failed to create tray icon from image data")
}

#[tokio::main]
async fn main() {
    let mut app_config_raw = config::AppConfig::load_or_create();
    let ptt_enabled_state = Arc::new(AtomicBool::new(false));

    // Initial Discord Auth if needed (Blocking for simplicity in init)
    if app_config_raw.discord_access_token.is_none() {
        if let (Some(client_id), Some(client_secret)) = (app_config_raw.discord_client_id.clone(), app_config_raw.discord_client_secret.clone()) {
            println!("Starting Discord auth flow...");
            let mut auth_client = discord::DiscordClient::new(client_id.clone());
            let res: anyhow::Result<()> = async {
                auth_client.connect().await?;
                let code = auth_client.get_auth_code().await?;
                let token = discord::oauth::exchange_code(&client_id, &client_secret, &code).await?;
                app_config_raw.discord_access_token = Some(token);
                app_config_raw.save();
                Ok(())
            }.await;

            if let Err(e) = res {
                println!("OAUTH FAILED: {}", e);
            }
        }
    }

    let app_config = Arc::new(app_config_raw);
    let (tx_ui, rx_ui) = mpsc::unbounded_channel::<AppMessage>();
    let (tx_cmd, rx_cmd) = mpsc::unbounded_channel::<UICommand>();

    // Start Backend Engine
    let backend = core::backend::Backend::new(
        tx_ui.clone(),
        rx_cmd,
        (*app_config).clone(),
        ptt_enabled_state.clone(),
    );
    tokio::spawn(backend.run());

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
        "Oto",
        native_options,
        Box::new(move |cc| {
            let ui_ctx = cc.egui_ctx.clone();

            // Tray Setup
            let tray_menu = Menu::new();
            let quit_item = MenuItem::new("Quit", true, None);
            let _ = tray_menu.append(&quit_item);
            let quit_id = quit_item.id().clone();

            let tray_icon = TrayIconBuilder::new()
                .with_tooltip("Oto")
                .with_icon(load_tray_icon())
                .with_menu(Box::new(tray_menu)) 
                .build()
                .unwrap();

            // Tray Listeners
            let tx_tray = tx_ui.clone();
            let ctx_tray = ui_ctx.clone();
            std::thread::spawn(move || {
                let tray_rx = TrayIconEvent::receiver();
                let menu_rx = MenuEvent::receiver();
                
                loop {
                    if let Ok(event) = tray_rx.try_recv() {
                        if let TrayIconEvent::Click { button, button_state, .. } = event {
                            if button == MouseButton::Left && button_state == MouseButtonState::Up {
                                let _ = tx_tray.send(AppMessage::ToggleOverlay);
                                ctx_tray.request_repaint();
                            }
                        }
                    }
                    if let Ok(event) = menu_rx.try_recv() {
                        if event.id == quit_id {
                            std::process::exit(0);
                        }
                    }
                    std::thread::sleep(Duration::from_millis(100));
                }
            });

            Ok(Box::new(ui::MixerApp::new(rx_ui, tx_cmd, tray_icon, app_config, ptt_enabled_state.clone())))
        }),
    );
}
