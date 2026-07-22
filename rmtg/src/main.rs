#![windows_subsystem = "windows"]
use bevy::app::AppExit;
use fdlimit::raise_fd_limit;
fn main() -> AppExit {
    raise_fd_limit().unwrap();
    rmtg_lib::app::app_run()
}
