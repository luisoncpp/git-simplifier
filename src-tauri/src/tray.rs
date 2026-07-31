//! Always-on system tray: close hides the window; Quit from the tray exits.

use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{
    App, AppHandle, Manager, Window, WindowEvent,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};

const MAIN_WINDOW: &str = "main";
const SHOW_ID: &str = "show";
const QUIT_ID: &str = "quit";

/// When true, `CloseRequested` is allowed to destroy the window so Quit can exit.
pub struct ExitAllowed(AtomicBool);

impl ExitAllowed {
    pub fn new() -> Self {
        Self(AtomicBool::new(/*allowed=*/false))
    }

    pub fn allow(&self) {
        self.0.store(/*allowed=*/true, Ordering::SeqCst);
    }

    pub fn is_allowed(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}

pub fn install(app: &App) -> tauri::Result<()> {
    let show = MenuItem::with_id(
        app,
        SHOW_ID,
        "Show",
        /*enabled=*/true,
        None::<&str>,
    )?;
    let quit = MenuItem::with_id(
        app,
        QUIT_ID,
        "Quit",
        /*enabled=*/true,
        None::<&str>,
    )?;
    let menu = Menu::with_items(app, &[&show, &quit])?;
    let icon = app
        .default_window_icon()
        .expect("bundled window icon")
        .clone();

    let _tray = TrayIconBuilder::with_id("main-tray")
        .icon(icon)
        .tooltip("Git Simplifier")
        .menu(&menu)
        .show_menu_on_left_click(/*show=*/false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            SHOW_ID => show_main(app),
            QUIT_ID => quit_app(app),
            _ => {}
        })
        .on_tray_icon_event(/*restore on left click*/ |tray, event| {
            if is_left_click_up(&event) {
                show_main(tray.app_handle());
            }
        })
        .build(app)?;

    Ok(())
}

pub fn on_window_event(window: &Window, event: &WindowEvent) {
    let WindowEvent::CloseRequested { api, .. } = event else {
        return;
    };
    let allowed = window
        .state::<ExitAllowed>()
        .is_allowed();
    if allowed {
        return;
    }
    let _ = window.hide();
    api.prevent_close();
}

fn show_main(app: &AppHandle) {
    let Some(window) = app.get_webview_window(MAIN_WINDOW) else {
        return;
    };
    let _ = window.unminimize();
    let _ = window.show();
    let _ = window.set_focus();
}

fn quit_app(app: &AppHandle) {
    app.state::<ExitAllowed>().allow();
    app.exit(0);
}

fn is_left_click_up(event: &TrayIconEvent) -> bool {
    matches!(
        event,
        TrayIconEvent::Click {
            button: MouseButton::Left,
            button_state: MouseButtonState::Up,
            ..
        }
    )
}
