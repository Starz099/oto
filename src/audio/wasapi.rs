use windows::core::{Interface, PWSTR};
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
use crate::core::AudioError;

pub struct WasapiManager;

impl WasapiManager {
    pub fn new() -> Self {
        Self
    }

    pub fn get_active_sessions(&self) -> std::result::Result<Vec<AudioProcess>, AudioError> {
        let mut sessions_list = Vec::new();

        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)
                .map_err(AudioError::Com)?;
            let device = enumerator.GetDefaultAudioEndpoint(eRender, eMultimedia)
                .map_err(AudioError::Com)?;

            let session_manager: IAudioSessionManager2 = device.Activate(CLSCTX_ALL, None)
                .map_err(AudioError::Com)?;
            
            let session_enumerator = session_manager.GetSessionEnumerator()
                .map_err(AudioError::Com)?;
            let session_count = session_enumerator.GetCount()
                .map_err(AudioError::Com)?;

            let endpoint_volume: IAudioEndpointVolume = device.Activate(CLSCTX_ALL, None)
                .map_err(AudioError::Com)?;
            let master_vol = endpoint_volume.GetMasterVolumeLevelScalar()
                .map_err(AudioError::Com)?;
            
            sessions_list.push(AudioProcess {
                pid: 0,
                name: "System Master".to_string(),
                volume: master_vol * 100.0,
            });

            for i in 0..session_count {
                if let Ok(session) = session_enumerator.GetSession(i) {
                    if let Ok(simple_volume) = session.cast::<ISimpleAudioVolume>() {
                        if let Ok(current_vol) = simple_volume.GetMasterVolume() {
                            if let Ok(control) = session.cast::<IAudioSessionControl2>() {
                                if let Ok(pid) = control.GetProcessId() {
                                    if pid != 0 {
                                        let raw_name = get_process_name(pid).unwrap_or_else(|| format!("Unknown ({})", pid));
                                        let normalized_name = normalize_process_name(&raw_name);
                                        sessions_list.push(AudioProcess {
                                            pid,
                                            name: normalized_name,
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

    pub fn set_process_volume(&self, target_pid: u32, volume_percent: f32) -> std::result::Result<(), AudioError> {
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)
                .map_err(AudioError::Com)?;
            let device = enumerator.GetDefaultAudioEndpoint(eRender, eMultimedia)
                .map_err(AudioError::Com)?;

            if target_pid == 0 {
                let endpoint_volume: IAudioEndpointVolume = device.Activate(CLSCTX_ALL, None)
                    .map_err(AudioError::Com)?;
                return endpoint_volume.SetMasterVolumeLevelScalar(volume_percent / 100.0, std::ptr::null())
                    .map_err(AudioError::Com);
            }

            let session_manager: IAudioSessionManager2 = device.Activate(CLSCTX_ALL, None)
                .map_err(AudioError::Com)?;
            let session_enumerator = session_manager.GetSessionEnumerator()
                .map_err(AudioError::Com)?;
            let session_count = session_enumerator.GetCount()
                .map_err(AudioError::Com)?;

            for i in 0..session_count {
                if let Ok(session) = session_enumerator.GetSession(i) {
                    if let Ok(control) = session.cast::<IAudioSessionControl2>() {
                        if let Ok(pid) = control.GetProcessId() {
                            if pid == target_pid {
                                if let Ok(simple_volume) = session.cast::<ISimpleAudioVolume>() {
                                    simple_volume.SetMasterVolume(volume_percent / 100.0, std::ptr::null())
                                        .map_err(AudioError::Com)?;
                                    return Ok(());
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

fn normalize_process_name(name: &str) -> String {
    if name.to_lowercase().ends_with(".exe") {
        let stem = &name[..name.len() - 4];
        let mut chars = stem.chars();
        match chars.next() {
            None => stem.to_string(),
            Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
        }
    } else {
        name.to_string()
    }
}

fn get_process_name(pid: u32) -> Option<String> {
    unsafe {
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

pub struct PersistentMic {
    endpoint_volume: IAudioEndpointVolume,
}

unsafe impl Send for PersistentMic {}
unsafe impl Sync for PersistentMic {}

impl PersistentMic {
    pub fn new() -> std::result::Result<Self, AudioError> {
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)
                .map_err(AudioError::Com)?;
            let device = enumerator.GetDefaultAudioEndpoint(eCapture, eConsole)
                .map_err(AudioError::Com)?;
            let endpoint_volume: IAudioEndpointVolume = device.Activate(CLSCTX_ALL, None)
                .map_err(AudioError::Com)?;
            
            Ok(Self { endpoint_volume })
        }
    }

    pub fn set_mute(&self, mute: bool) -> std::result::Result<(), AudioError> {
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            self.endpoint_volume.SetMute(mute, std::ptr::null())
                .map_err(AudioError::Com)?;
        }
        Ok(())
    }

    pub fn refresh(&mut self) -> std::result::Result<(), AudioError> {
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)
                .map_err(AudioError::Com)?;
            let device = enumerator.GetDefaultAudioEndpoint(eCapture, eConsole)
                .map_err(AudioError::Com)?;
            let endpoint_volume: IAudioEndpointVolume = device.Activate(CLSCTX_ALL, None)
                .map_err(AudioError::Com)?;
            
            self.endpoint_volume = endpoint_volume;
        }
        Ok(())
    }
}
