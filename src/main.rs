#![windows_subsystem = "windows"]

mod app;
mod event_loop;
mod settings;
mod system;
mod theme;
mod widgets;

use app::ShowcaseApp;
use eframe::egui;
use system::{SingleInstanceGuard, TrayController, load_window_icon};

fn main() -> eframe::Result<()> {
    let Some(_guard) = SingleInstanceGuard::try_acquire() else {
        return Ok(());
    };

    let start_minimized = std::env::args().any(|a| a.eq_ignore_ascii_case("--minimized"));
    let (tray, tray_rx) = TrayController::start();

    let mut viewport = egui::ViewportBuilder::default()
        .with_title("澄明界面展示")
        .with_inner_size([1180.0, 760.0])
        .with_min_inner_size([820.0, 540.0])
        .with_visible(!start_minimized);

    if let Some(icon) = load_window_icon() {
        viewport = viewport.with_icon(icon);
    }

    let options = eframe::NativeOptions {
        viewport,
        renderer: eframe::Renderer::Glow,
        centered: true,
        ..Default::default()
    };

    event_loop::run(
        "澄明界面展示",
        options,
        Box::new(move |cc| Ok(Box::new(ShowcaseApp::new(cc, tray, tray_rx)))),
    )
}
