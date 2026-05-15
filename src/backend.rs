use std::fs::OpenOptions;
use std::io::{Read, Write};
use sysinfo::System;
use windows::core::Interface;
use windows::Win32::System::Com::{CoInitializeEx, CoCreateInstance, COINIT_MULTITHREADED, CLSCTX_ALL};
use windows::Win32::Media::Audio::{
    IMMDeviceEnumerator, MMDeviceEnumerator, eRender, eConsole, 
    IAudioSessionManager2, IAudioSessionEnumerator, 
    IAudioSessionControl2, ISimpleAudioVolume
};
use serde_json::Value;

fn ui_to_api_vol(ui_percentage: f64) -> f64 {
    // Converts human-readable UI % to raw API amplitude
    100.0 * (ui_percentage / 100.0).powf(2.77)
}

fn send_frame(pipe: &mut std::fs::File, opcode: u32, payload: &str) {
    let bytes = payload.as_bytes();
    let length = bytes.len() as u32;
    if pipe.write_all(&opcode.to_le_bytes()).is_ok() && pipe.write_all(&length.to_le_bytes()).is_ok() {
        let _ = pipe.write_all(bytes);
    }
}

fn read_frame(pipe: &mut std::fs::File) -> String {
    let mut header = [0u8; 8];
    if pipe.read_exact(&mut header).is_err() {
        return String::from("{}"); 
    }
    
    let length = u32::from_le_bytes(header[4..8].try_into().unwrap_or([0,0,0,0]));
    if length == 0 {
        return String::from("{}");
    }

    let mut payload = vec![0u8; length as usize];
    if pipe.read_exact(&mut payload).is_err() {
        return String::from("{}");
    }
    
    String::from_utf8(payload).unwrap_or_else(|_| String::from("{}"))
}

// AUTHENTICATION MIDDLEWARE
async fn exchange_code_locally(auth_code: &str, client_id: &str, client_secret: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let params = format!(
        "client_id={}&client_secret={}&grant_type=authorization_code&code={}&redirect_uri=http://127.0.0.1",
        client_id, client_secret, auth_code
    );

    let res = client.post("https://discord.com/api/oauth2/token")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(params)
        .send()
        .await?;
        
    let json: Value = res.json().await?;
    if let Some(token) = json["access_token"].as_str() {
        Ok(token.to_string())
    } else {
        Err(format!("Token exchange failed: {}", json).into())
    }
}

#[tokio::main]
async fn main() -> windows::core::Result<()> {
    let client_id = "1498438876065435788";
    let client_secret = "2LzSHIT-7g4UWOJV8l2svoargE5Jb-q3";
    
    let mut sys = System::new_all();
    sys.refresh_all();

    let pipe_path = r"\\.\pipe\discord-ipc-0";
    
    if let Ok(mut pipe) = OpenOptions::new().read(true).write(true).open(pipe_path) {
        println!("Connected to Discord Pipe. Authenticating...\n");
        
        send_frame(&mut pipe, 0, &format!(r#"{{"v": 1, "client_id": "{}"}}"#, client_id));
        let _ = read_frame(&mut pipe); 

        send_frame(&mut pipe, 1, &format!(r#"{{
            "cmd": "AUTHORIZE",
            "args": {{ 
                "client_id": "{}", 
                "scopes": ["rpc", "rpc.voice.read", "rpc.voice.write"]
            }},
            "nonce": "auth_req_1"
        }}"#, client_id));
        
        let raw_auth_response = loop {
            let frame = read_frame(&mut pipe);
            
            if frame.contains(r#""nonce":"auth_req_1""#) {
                break frame;
            } else if frame == "{}" {
                panic!("Pipe closed unexpectedly during authorization.");
            }
        };
        
        let auth_res: Value = serde_json::from_str(&raw_auth_response).unwrap_or_default();
        
        if let Some(code) = auth_res["data"]["code"].as_str() {
            println!("Exchanging Auth Code for Token...");

            // AUTHENTICATE ENGINE
            if let Ok(token) = exchange_code_locally(code, client_id, client_secret).await {
                send_frame(&mut pipe, 1, &format!(r#"{{"cmd": "AUTHENTICATE", "args": {{ "access_token": "{}" }}, "nonce": "auth_req_2"}}"#, token));
                
                loop {
                    let frame = read_frame(&mut pipe);
                    if frame.contains(r#""nonce":"auth_req_2""#) { break; }
                }
                println!("[SUCCESS] Discord Engine Unlocked!\n");

                // DISCORD INTERNAL MIXER DATA (Self)
                send_frame(&mut pipe, 1, r#"{"cmd": "GET_VOICE_SETTINGS", "nonce": "voice_1"}"#);
                loop {
                    let frame = read_frame(&mut pipe);
                    if frame.contains(r#""nonce":"voice_1""#) {
                        if let Ok(voice_json) = serde_json::from_str::<Value>(&frame) {
                            if let Some(data) = voice_json.get("data") {
                                println!("=== Discord Internal Mixer (Self) ===");
                                println!("-> Mic Volume    : {:.0}%", data["input"]["volume"].as_f64().unwrap_or(0.0));
                                println!("-> Output Volume : {:.0}%", data["output"]["volume"].as_f64().unwrap_or(0.0));
                                println!("-> Muted         : {}\n", data["mute"].as_bool().unwrap_or(false));
                            }
                        }
                        break;
                    }
                }

                // SCAN AND FORMAT ACTIVE VOICE CHANNEL
                send_frame(&mut pipe, 1, r#"{"cmd": "GET_SELECTED_VOICE_CHANNEL", "nonce": "vc_1"}"#);
                loop {
                    let frame = read_frame(&mut pipe);
                    if frame.contains(r#""nonce":"vc_1""#) {
                        if let Ok(vc_json) = serde_json::from_str::<Value>(&frame) {
                            let vc_name = vc_json["data"]["name"].as_str().unwrap_or("Unknown VC");
                            println!("=== Active VC: {} ===", vc_name);
                            
                            if let Some(states) = vc_json["data"]["voice_states"].as_array() {
                                for user in states {
                                    let name = user["nick"].as_str().unwrap_or(user["user"]["username"].as_str().unwrap_or("Unknown"));
                                    let vol = user["volume"].as_f64().unwrap_or(0.0);
                                    let id = user["user"]["id"].as_str().unwrap_or("Unknown");
                                    println!("-> {} (ID: {}) | Volume: {:.1}%", name, id, vol);
                                }
                            }
                        }
                        println!();
                        break;
                    }
                }

                println!("=== Changing Friend's Volume ===");
                
                let target_user_id = "811120162656616448"; 
                let new_volume = 20.0;
                
                let converted_api_volume = ui_to_api_vol(new_volume);

                let set_vol_cmd = format!(r#"{{
                    "cmd": "SET_USER_VOICE_SETTINGS",
                    "args": {{
                        "user_id": "{}",
                        "volume": {}
                    }},
                    "nonce": "set_vol_1"
                }}"#, target_user_id, converted_api_volume);
                
                send_frame(&mut pipe, 1, &set_vol_cmd);
                
                loop {
                    let frame = read_frame(&mut pipe);
                    if frame.contains(r#""nonce":"set_vol_1""#) {
                        println!("Successfully forced dainik bhaskar's volume to {}%\n", new_volume);
                        break;
                    }
                }

            } else {
                println!("[FAILED] HTTP Token Exchange Failed.");
            }
        } else {
            println!("\n[!] CRASH AVOIDED: Discord did not return a code. It returned: {}", raw_auth_response);
        }
    } else {
        println!("[WARNING] Discord pipe not found. Ensure Discord is running.");
    }

    println!("=== OS Master Mixer ===");
    unsafe {
        CoInitializeEx(None, COINIT_MULTITHREADED).ok()?;
        let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
        let device = enumerator.GetDefaultAudioEndpoint(eRender, eConsole)?;
        let session_manager: IAudioSessionManager2 = device.Activate::<IAudioSessionManager2>(CLSCTX_ALL, None)?;
        let session_enumerator: IAudioSessionEnumerator = session_manager.GetSessionEnumerator()?;
        let count = session_enumerator.GetCount()?;

        for i in 0..count {
            if let Ok(session) = session_enumerator.GetSession(i) {
                if let Ok(simple_volume) = session.cast::<ISimpleAudioVolume>() {
                    let current_volume = simple_volume.GetMasterVolume().unwrap_or(0.0);
                    if let Ok(session2) = session.cast::<IAudioSessionControl2>() {
                        let pid = session2.GetProcessId().unwrap_or(0);
                        let proc_name = if pid == 0 { "System Sounds".to_string() } else {
                            sys.process(sysinfo::Pid::from_u32(pid))
                               .map(|p| p.name().to_str().unwrap_or("Unknown"))
                               .unwrap_or("Unknown").to_string()
                        };
                        println!("=> {} (PID: {}) | {:.0}%", proc_name, pid, current_volume * 100.0);
                    }
                }
            }
        }
    }
    
    Ok(())
}