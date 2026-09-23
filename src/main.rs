mod auth;
mod icons;
mod ui;
mod crypto;
mod db;
mod models;
mod os_key;
mod provider;
mod system_env;

use eframe::egui;

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([960.0, 640.0])
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
