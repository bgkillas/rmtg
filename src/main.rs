#![windows_subsystem = "windows"]
use bevy::app::AppExit;
pub fn main() -> AppExit {
    #[cfg(target_os = "windows")]
    unsafe {
        winapi::um::wincon::AttachConsole(winapi::um::wincon::ATTACH_PARENT_PROCESS);
    }
    rmtg::start()
}
