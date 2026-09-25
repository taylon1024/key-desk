use eframe::egui;

use crate::models::Variable;

use super::KeyDesk;
use super::theme::{self, hairline};

impl KeyDesk {
    pub(super) fn show_list(&mut self, ui: &mut egui::Ui) {
        let mut scopes: Vec<String> = self
            .variables
            .iter()
            .map(|variable| variable.scope.clone())
            .collect();
        scopes.sort();
        scopes.dedup();

        theme::ink_bar(ui, "STORED ENTRIES", "LOCAL VAULT");
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.monospace("scope");
            let selected = if self.scope_filter.is_empty() {
                "ALL".to_string()
            } else {
                self.scope_filter.clone()
            };
            let response = egui::ComboBox::from_id_salt("scope-filter")
                .selected_text(&selected)
                .width(160.0)
                .truncate()
                .show_ui(ui, |ui| {
                    ui.set_min_width(260.0);
                    if ui
                        .add_sized(
                            [240.0, 24.0],
                            egui::Button::selectable(self.scope_filter.is_empty(), "ALL"),
                        )
                        .clicked()
                    {
                        self.scope_filter.clear();
                    }
                    for scope in &scopes {
                        if ui
                            .add_sized(
                                [240.0, 24.0],
                                egui::Button::selectable(self.scope_filter == *scope, scope)
                                    .truncate(),
                            )
                            .on_hover_text(scope)
                            .clicked()
                        {
                            self.scope_filter = scope.clone();
                        }
                    }
                });
            response.response.on_hover_text(&selected);
            if ui.button("EXPORT").clicked() {
                self.export(ui.ctx());
            }
            if ui.button("IMPORT").clicked() {
                self.open_env_import();
            }
            let count = self
                .variables
                .iter()
                .filter(|variable| {
                    self.scope_filter.is_empty() || variable.scope == self.scope_filter
                })
                .count();
            ui.monospace(format!("{count:02}"));
        });
        ui.add_space(4.0);
        hairline(ui);

        let visible: Vec<Variable> = self
            .variables
            .iter()
            .filter(|variable| self.scope_filter.is_empty() || variable.scope == self.scope_filter)
            .cloned()
            .collect();

        egui::ScrollArea::vertical()
            .max_height(160.0)
            .min_scrolled_height(160.0)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for variable in visible {
                    let shown = self.revealed.contains(&variable.id);
                    let preview = if !shown {
                        "••••".to_string()
                    } else if variable.value.is_empty() {
                        "empty".to_string()
                    } else if variable.value.chars().count() > 18 {
                        let head: String = variable.value.chars().take(18).collect();
                        format!("{head}…")
                    } else {
                        variable.value.clone()
                    };
                    ui.horizontal(|ui| {
                        let key_width = 184.0;
                        let scope_width = 140.0;
                        let provider = crate::provider::provider_label_for_key(&variable.key);
                        let icon_width = if provider.is_some() {
                            16.0 + ui.spacing().item_spacing.x
                        } else {
                            0.0
                        };
                        let preview_width = (ui.available_width()
                            - key_width
                            - scope_width
                            - icon_width
                            - ui.spacing().item_spacing.x * 2.0)
                            .max(80.0);
                        if let Some(label) = provider {
                            theme::provider_icon(ui, label);
                        }
                        theme::ink_badge(ui, &variable.key, key_width).on_hover_text(&variable.key);
                        ui.add_sized(
                            [preview_width, 24.0],
                            egui::Label::new(egui::RichText::new(preview).monospace()).truncate(),
                        );
                        ui.add_sized(
                            [scope_width, 24.0],
                            egui::Label::new(egui::RichText::new(&variable.scope).monospace())
                                .truncate(),
                        )
                        .on_hover_text(&variable.scope);
                    });
                    ui.horizontal(|ui| {
                        if ui
                            .small_button(if shown { "HIDE" } else { "SHOW" })
                            .clicked()
                        {
                            if shown {
                                self.revealed.remove(&variable.id);
                            } else {
                                self.revealed.insert(variable.id);
                            }
                        }
                        if ui.small_button("COPY").clicked() {
                            ui.ctx()
                                .copy_text(crate::models::format_assignment(&variable));
                            self.message = format!("copied {}", variable.key);
                        }
                        if ui.small_button("EDIT").clicked() {
                            self.start_edit(&variable);
                        }
                        if ui.small_button("DEL").clicked() {
                            self.delete(variable.id);
                        }
                        if !variable.description.is_empty() {
                            ui.add_sized(
                                [ui.available_width(), 20.0],
                                egui::Label::new(
                                    egui::RichText::new(&variable.description).monospace(),
                                )
                                .truncate(),
                            );
                        }
                    });
                    hairline(ui);
                }
            });
    }
}

#[cfg(test)]
#[test]
fn list_frame_height_is_stable_when_entries_change() {
    let mut heights = Vec::new();
    egui::__run_test_ui(|ui| {
        ui.set_width(600.0);
        for count in [0, 1, 10] {
            let mut app = KeyDesk::locked();
            app.variables = (0..count)
                .map(|id| Variable {
                    id,
                    key: format!("KEY_{id}"),
                    value: "value".to_string(),
                    scope: "a-very-long-scope-name-for-layout".to_string(),
                    description: String::new(),
                    is_secret: false,
                    created_at: String::new(),
                    updated_at: String::new(),
                })
                .collect();
            let frame = theme::ruled_frame(ui.ctx()).show(ui, |ui| app.show_list(ui));
            heights.push(frame.response.rect.height());
        }
    });
    for pass in heights.chunks_exact(3) {
        assert!(pass.iter().all(|height| *height == pass[0]), "{pass:?}");
    }
}
