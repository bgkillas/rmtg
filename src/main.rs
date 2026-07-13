#![windows_subsystem = "windows"]
use bevy::app::AppExit;
fn main() -> AppExit {
    rmtg_lib::app::app_run()
}
