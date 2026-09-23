mod app;
mod db;
mod models;
mod provider;
mod system_env;

use eframe::egui;

fn main() -> eframe::Result {
    let db_path = db::db_path();
    let connection = match db::open(&db_path) {
        Ok(connection) => connection,
        Err(err) => {
            eprintln!("无法打开数据库 {}: {err}", db_path.display());
            std::process::exit(1);
        }
    };

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
            app::install_cjk_font(&cc.egui_ctx);
            Ok(Box::new(app::KeyDesk::new(connection)))
        }),
    )
}
