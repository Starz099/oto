mod app;
mod ui;
mod audio;

use tokio::sync::mpsc;
use crate::app::{AppMessage, UICommand};
use std::time::Duration;

#[tokio::main]
async fn main() {
    let (tx, rx) = mpsc::unbounded_channel::<AppMessage>();
    let (tx_cmd, mut rx_cmd) = mpsc::unbounded_channel::<UICommand>();

    //Polling for active sessions every 1 second in the background
    let tx_clone = tx.clone();
    tokio::spawn(async move {
        println!("Background Poller started. Fetching sessions every 1 second...");
        
        loop {
            if let Ok(actual_sessions) = audio::wasapi::get_active_sessions() {
                let _ = tx_clone.send(AppMessage::UpdateSessions(actual_sessions));
            }            
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    });


    tokio::spawn(async move {
        // println!("Backend engine started. Waiting 3 seconds...");
        // tokio::time::sleep(Duration::from_secs(3)).await;
        
        // println!("Sending command to update volume to 100!");
        // let _ = tx.send(app::AppMessage::UpdateStarzVolume(100.0));

        println!("Backend Engine started. Listening for UI commands...");
        tokio::time::sleep(Duration::from_millis(500)).await;

        if let Ok(actual_sessions) = audio::wasapi::get_active_sessions() {
            let _ = tx.send(app::AppMessage::UpdateSessions(actual_sessions));
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

    ui::run_overlay(rx, tx_cmd);
}