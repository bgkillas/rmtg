#![windows_subsystem = "windows"]
pub fn main() {
    #[cfg(target_os = "windows")]
    unsafe {
        winapi::um::wincon::AttachConsole(winapi::um::wincon::ATTACH_PARENT_PROCESS);
    }
    rmtg::start()
}
