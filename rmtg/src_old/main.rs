#![windows_subsystem = "windows"]
use bevy::app::AppExit;
pub fn main() -> AppExit {
    #[cfg(target_os = "windows")]
    unsafe {
        winapi::um::wincon::AttachConsole(winapi::um::wincon::ATTACH_PARENT_PROCESS);
    }
    //generate_mips()
    rmtg::start()
}
/*
macro_rules! generate_mips {
    [$(($a:literal, $b:literal)),*] => {
        $(
            let bytes = include_bytes!($a);
            let mut image = rmtg::download::parse_bytes(bytes).unwrap();
            rmtg::download::write_file($b, &mut image);
        )*
    };
}
#[allow(dead_code)]
pub fn generate_mips() -> AppExit {
    generate_mips![("../assets/back.jpg", "./assets/back.mip")];
    AppExit::Success
}
*/
