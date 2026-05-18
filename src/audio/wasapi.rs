use windows::core::{Interface, Result, PWSTR};
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
use windows::Win32::Media::Audio::{
    eMultimedia, eRender, IAudioSessionControl2, eCapture, eConsole,
    IAudioSessionManager2, ISimpleAudioVolume, IMMDeviceEnumerator, MMDeviceEnumerator,
};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_MULTITHREADED,
};
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};
use crate::app::AudioProcess;

// Fetch Active Audio Sessions
pub fn get_active_sessions() -> Result<Vec<AudioProcess>> {
    let mut sessions_list = Vec::new();

    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED).ok();

        // Get Audio Device Enumerator
        let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
        let device = enumerator.GetDefaultAudioEndpoint(eRender, eMultimedia)?;

        // Activate Audio Session Manager
        let session_manager: IAudioSessionManager2 = device.Activate(CLSCTX_ALL, None)?;
        
        // Get the Session Enumerator (List of all active audio apps)
        let session_enumerator = session_manager.GetSessionEnumerator()?;
        let session_count = session_enumerator.GetCount()?;

        // Add System Master Volume as PID 0
        let endpoint_volume: IAudioEndpointVolume = device.Activate(CLSCTX_ALL, None)?;
        let master_vol = endpoint_volume.GetMasterVolumeLevelScalar()?;
        sessions_list.push(AudioProcess {
            pid: 0,
            name: "System Master".to_string(),
            volume: master_vol * 100.0,
        });

        // Loop through all active apps
        for i in 0..session_count {
            if let Ok(session) = session_enumerator.GetSession(i) {
                // Get the SimpleAudioVolume interface to read volume
                if let Ok(simple_volume) = session.cast::<ISimpleAudioVolume>() {
                    // Naya Rustic tareeka: direct return catch karo
                    if let Ok(current_vol) = simple_volume.GetMasterVolume() {
                        
                        // We need IAudioSessionControl2 to get the Process ID (PID)
                        if let Ok(control) = session.cast::<IAudioSessionControl2>() {
                            
                            if let Ok(pid) = control.GetProcessId() {
                                if pid != 0 {
                                    // Fetch the human-readable .exe name using the PID
                                    let name = get_process_name(pid).unwrap_or_else(|| format!("Unknown ({})", pid));
                                    
                                    sessions_list.push(AudioProcess {
                                        pid,
                                        name,
                                        volume: current_vol * 100.0,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(sessions_list)
}

// Set Volume for a Specific App or System
pub fn set_process_volume(target_pid: u32, volume_percent: f32) -> Result<()> {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED).ok();
        let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
        let device = enumerator.GetDefaultAudioEndpoint(eRender, eMultimedia)?;

        // If PID is 0, update Master System Volume
        if target_pid == 0 {
            let endpoint_volume: IAudioEndpointVolume = device.Activate(CLSCTX_ALL, None)?;
            return endpoint_volume.SetMasterVolumeLevelScalar(volume_percent / 100.0, std::ptr::null());
        }

        // Otherwise, find the specific app and change its volume
        let session_manager: IAudioSessionManager2 = device.Activate(CLSCTX_ALL, None)?;
        let session_enumerator = session_manager.GetSessionEnumerator()?;
        let session_count = session_enumerator.GetCount()?;

        for i in 0..session_count {
            if let Ok(session) = session_enumerator.GetSession(i) {
                if let Ok(control) = session.cast::<IAudioSessionControl2>() {
                    if let Ok(pid) = control.GetProcessId() {
                        if pid == target_pid {
                            if let Ok(simple_volume) = session.cast::<ISimpleAudioVolume>() {
                                simple_volume.SetMasterVolume(volume_percent / 100.0, std::ptr::null())?;
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

// Helper: Convert PID to .exe name
fn get_process_name(pid: u32) -> Option<String> {
    unsafe {
        // Open the process with limited info rights
        let handle: HANDLE = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        
        let mut buffer = [0u16; 1024];
        let mut size = buffer.len() as u32;
        
        if QueryFullProcessImageNameW(handle, PROCESS_NAME_WIN32, PWSTR(buffer.as_mut_ptr()), &mut size).is_ok() {
            let _ = CloseHandle(handle);
            let full_path = String::from_utf16_lossy(&buffer[..size as usize]);
            if let Some(exe_name) = full_path.split('\\').last() {
                return Some(exe_name.to_string());
            }
        }
        let _ = CloseHandle(handle);
        None
    }
}

/// Toggles the mute state of the Default Windows Microphone
pub fn set_default_mic_mute(mute: bool) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    unsafe {
        let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
        // eCapture specifies we want the Microphone, not the Speakers
        let device = enumerator.GetDefaultAudioEndpoint(eCapture, eConsole)?;
        let endpoint_volume: IAudioEndpointVolume = device.Activate(CLSCTX_ALL, None)?;
        
        endpoint_volume.SetMute(mute, std::ptr::null())?;
    }
    Ok(())
}


pub struct PersistentMic {
    endpoint_volume: IAudioEndpointVolume,
}

// Safely tell Rust we can share this COM pointer across our Tokio threads
unsafe impl Send for PersistentMic {}
unsafe impl Sync for PersistentMic {}

impl PersistentMic {
    pub fn new() -> std::result::Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        unsafe {
            // Initialize MTA (Multi-Threaded Apartment) for background engine
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            
            let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
            let device = enumerator.GetDefaultAudioEndpoint(eCapture, eConsole)?;
            let endpoint_volume: IAudioEndpointVolume = device.Activate(CLSCTX_ALL, None)?;
            
            Ok(Self { endpoint_volume })
        }
    }

    pub fn set_mute(&self, mute: bool) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        unsafe {
            self.endpoint_volume.SetMute(mute, std::ptr::null())?;
        }
        Ok(())
    }

    pub fn refresh(&mut self) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        unsafe {
            let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
            let device = enumerator.GetDefaultAudioEndpoint(eCapture, eConsole)?;
            let endpoint_volume: IAudioEndpointVolume = device.Activate(CLSCTX_ALL, None)?;
            
            // Overwrite the old pointer with the fresh one
            self.endpoint_volume = endpoint_volume;
        }
        Ok(())
    }
}