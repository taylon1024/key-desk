mod auth;
mod crypto;
mod logging;
#[cfg(target_os = "macos")]
mod menu_bar;
mod db;
mod icons;
mod models;
mod provider;
mod system_env;
mod ui;

use eframe::egui;

fn main() -> eframe::Result {
    logging::init();
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([700.0, 760.0])
            .with_min_inner_size([640.0, 740.0])
            .with_title("key-desk"),
        ..Default::default()
    };

    eframe::run_native(
        "key-desk",
        native_options,
        Box::new(move |cc| {
            ui::install_cjk_font(&cc.egui_ctx);
            Ok(Box::new(ui::KeyDesk::locked()))
        }),
    )
}
