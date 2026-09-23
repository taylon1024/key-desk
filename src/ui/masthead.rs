use eframe::egui;

use super::theme::{self, pixel_mark, ruled_frame};
use super::KeyDesk;

impl KeyDesk {
    pub(super) fn show_masthead(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            pixel_mark(ui, 52.0);
            ui.add_space(8.0);
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("KEYDESK").monospace().size(28.0));
                ui.monospace("local keys");
                ui.monospace("encrypted at rest");
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                ruled_frame().show(ui, |ui| {
                    ui.set_min_width(168.0);
                    ui.label(egui::RichText::new("VAULT").monospace().size(22.0));
                    ui.monospace("aes-256-gcm");
                    ui.horizontal(|ui| {
                        theme::ink_badge(ui, "AES");
                        theme::ink_badge(ui, "ENV");
                    });
                });
            });
        });
    }
}
