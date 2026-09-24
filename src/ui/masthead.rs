use eframe::egui;

use super::KeyDesk;
use super::theme::pixel_mark;

impl KeyDesk {
    pub(super) fn show_masthead(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            pixel_mark(ui, 52.0);
            ui.add_space(12.0);
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("KEYDESK").monospace().size(28.0));
                ui.monospace("LOCAL / ENCRYPTED / FAST");
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.horizontal(|ui| self.show_theme_picker(ui));
            });
        });
    }
}
