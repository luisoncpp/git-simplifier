// Release builds must not attach a console, or Windows opens a terminal window
// behind the app. Debug builds keep it so `println!` diagnostics stay visible.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    git_helper_lib::run()
}
