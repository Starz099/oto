use sysinfo::System;
use windows::core::Interface;
use windows::Win32::System::Com::{CoInitializeEx, CoCreateInstance, COINIT_MULTITHREADED, CLSCTX_ALL};
use windows::Win32::Media::Audio::{IMMDeviceEnumerator, MMDeviceEnumerator, eRender, eConsole, IAudioSessionManager2, IAudioSessionEnumerator, IAudioSessionControl, IAudioSessionControl2, ISimpleAudioVolume};
fn main() -> windows::core::Result<()> {
    
    let mut sys = System::new_all();
    sys.refresh_all();

    unsafe {
        CoInitializeEx(None, COINIT_MULTITHREADED).ok()?;

        let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;


        let device = enumerator.GetDefaultAudioEndpoint(eRender, eConsole)?;
        println!("Successfully grabbed the default audio device!");
        println!("Device ID: {:?}", device.GetId()?);

        let session_manager: IAudioSessionManager2 = device.Activate::<IAudioSessionManager2>(CLSCTX_ALL, None)?;        
        let session_enumerator: IAudioSessionEnumerator = session_manager.GetSessionEnumerator()?;

        let count = session_enumerator.GetCount()?;
        
        println!("Total active audio apps found: {}", count);

        // let x = session_enumerator.GetSession(0).ok();
        // println!("First session: {:?}", x);

        // for i in 0..count {
        //     let session: IAudioSessionControl = session_enumerator.GetSession(i)?;
            
        //     if let Ok(session2) = session.cast::<IAudioSessionControl2>() {
        //         let pid = session2.GetProcessId()?;
                
        //         println!("Session {} - Process ID (PID): {}", i, pid);
        //     }
        // }

        for i in 0..count {
            let session: IAudioSessionControl = session_enumerator.GetSession(i)?;
            
            // 1. Volume Read Karo
            let mut current_volume: f32 = 0.0;
            if let Ok(simple_volume) = session.cast::<ISimpleAudioVolume>() {
                // GetMasterVolume memory me volume write kar deta hai
                current_volume = simple_volume.GetMasterVolume()?;
            }
            
            // 2. PID aur Naam Read Karo
            if let Ok(session2) = session.cast::<IAudioSessionControl2>() {
                let pid = session2.GetProcessId()?;
                
                if pid == 0 {
                    // PID 0 hamesha Windows System Sounds hota hai
                    println!("=> System Sounds | Volume: {:.0}%", current_volume * 100.0);
                } else {
                    // Sysinfo se process name nikalo
                    let mut proc_name = "Unknown App";
                    
                    // Rust me u32 (Process ID) ko sysinfo ke PID format me convert karke search kiya
                    if let Some(process) = sys.process(sysinfo::Pid::from_u32(pid)) {
                        proc_name = process.name().to_str().unwrap_or("Unknown App");
                    }
                    
                    println!("=> {} (PID: {}) | Volume: {:.0}%", proc_name, pid, current_volume * 100.0);
                }
            }
        }

    }
    
    println!("COM Library initialized successfully!");
    
    Ok(())
}