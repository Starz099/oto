mod app;
mod ui;

use tokio::sync::mpsc;
use crate::app::{UICommand, AppMessage};

#[tokio::main]
async fn main() {
    let (tx, rx) = mpsc::unbounded_channel::<AppMessage>();
    let (tx_cmd, mut rx_cmd) = mpsc::unbounded_channel::<UICommand>();

    tokio::spawn(async move {
        // println!("Backend engine started. Waiting 3 seconds...");
        // tokio::time::sleep(Duration::from_secs(3)).await;
        
        // println!("Sending command to update volume to 100!");
        // let _ = tx.send(app::AppMessage::UpdateStarzVolume(100.0));

        println!("Backend Engine started. Listening for UI commands...");
        while let Some(cmd) = rx_cmd.recv().await {
            match cmd {
                UICommand::SetProcessVolume { pid, volume } => {
                    println!("=> Woke up Tokio Thread! Instructing OS to set PID {} to {:.1}% volume", pid, volume);
                }
            }
        }
    });

    ui::run_overlay(rx, tx_cmd);
}