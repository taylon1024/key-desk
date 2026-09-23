use eframe::egui;

use crate::models::Variable;

use super::theme::{self, hairline};
use super::KeyDesk;

impl KeyDesk {
    pub(super) fn show_list(&mut self, ui: &mut egui::Ui) {
        let mut scopes: Vec<String> = self
            .variables
            .iter()
            .map(|variable| variable.scope.clone())
            .collect();
        scopes.sort();
        scopes.dedup();

        ui.horizontal(|ui| {
            ui.monospace("scope");
            egui::ComboBox::from_id_salt("scope-filter")
                .selected_text(if self.scope_filter.is_empty() {
                    "ALL"
                } else {
                    self.scope_filter.as_str()
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.scope_filter, String::new(), "ALL");
                    for scope in &scopes {
                        ui.selectable_value(&mut self.scope_filter, scope.clone(), scope);
                    }
                });
            if ui.button("EXPORT").clicked() {
                self.export();
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
            .max_height(280.0)
            .show(ui, |ui| {
                for variable in visible {
                    let shown = !variable.is_secret || self.revealed.contains(&variable.id);
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
                        theme::ink_badge(ui, &variable.key);
                        ui.monospace(format!("{:>8}", variable.scope));
                        theme::scale_tick(ui, theme::tick_at(&variable.key));
                        ui.monospace(preview);
                    });
                    ui.horizontal(|ui| {
                        if variable.is_secret
                            && ui.small_button(if shown { "HIDE" } else { "SHOW" }).clicked()
                        {
                            if shown {
                                self.revealed.remove(&variable.id);
                            } else {
                                self.revealed.insert(variable.id);
                            }
                        }
                        if ui.small_button("COPY").clicked() {
                            ui.ctx().copy_text(variable.value.clone());
                            self.message = format!("copied {}", variable.key);
                        }
                        if ui.small_button("EDIT").clicked() {
                            self.start_edit(&variable);
                        }
                        if ui.small_button("DEL").clicked() {
                            self.delete(variable.id);
                        }
                        if !variable.description.is_empty() {
                            ui.monospace(&variable.description);
                        }
                    });
                    hairline(ui);
                }
            });
    }
}
