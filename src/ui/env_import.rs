use eframe::egui;

use crate::models;
use crate::system_env::{self, EnvSource};

use super::KeyDesk;

struct EnvImportRow {
    key: String,
    value: String,
    importable: bool,
    selected: bool,
}

impl KeyDesk {
    pub(super) fn open_env_import(&mut self) {
        self.env_import_filter.clear();
        self.env_import_open = true;
        self.refresh_env_snapshot();
    }

    pub(super) fn refresh_env_snapshot(&mut self) {
        match system_env::list_env(self.env_import_source) {
            Ok(vars) => {
                self.env_load_error.clear();
                self.env_vars = vars;
                self.env_selected = self
                    .env_vars
                    .iter()
                    .filter(|v| v.importable)
                    .map(|v| v.key.clone())
                    .collect();
            }
            Err(err) => {
                self.env_load_error = err;
                self.env_vars.clear();
                self.env_selected.clear();
            }
        }
    }

    pub(super) fn run_env_import(&mut self) {
        let scope = self.env_import_scope.trim().to_string();
        if let Err(err) = models::validate_scope(&scope) {
            self.message = err;
            return;
        }
        let keys: Vec<String> = self.env_selected.iter().cloned().collect();
        let entries = self
            .env_vars
            .iter()
            .filter(|v| keys.contains(&v.key))
            .map(|v| (v.key.clone(), v.value.clone()));
        let report = system_env::import_into_db(
            self.db(),
            entries,
            &scope,
            self.env_import_source.import_description(),
        );
        self.reload();
        self.env_import_open = false;
        self.message = format!(
            "import done: {} saved, {} invalid, {} already exist",
            report.imported, report.skipped_invalid, report.skipped_conflict
        );
        if let Some(err) = report.errors.first() {
            self.message.push_str("; ");
            self.message.push_str(err);
        }
    }

    pub(super) fn show_env_import_window(&mut self, ctx: &egui::Context) {
        if !self.env_import_open {
            return;
        }
        let mut open = self.env_import_open;
        egui::Window::new("import env")
            .open(&mut open)
            .default_width(520.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("source");
                    let prev = self.env_import_source;
                    egui::ComboBox::from_id_salt("env-source")
                        .selected_text(self.env_import_source.label())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.env_import_source,
                                EnvSource::LoginShell,
                                EnvSource::LoginShell.label(),
                            );
                            ui.selectable_value(
                                &mut self.env_import_source,
                                EnvSource::Process,
                                EnvSource::Process.label(),
                            );
                        });
                    if self.env_import_source != prev {
                        self.refresh_env_snapshot();
                    }
                    if ui.button("refresh").clicked() {
                        self.refresh_env_snapshot();
                    }
                    ui.label(format!("{} vars", self.env_vars.len()));
                });
                ui.label(self.env_import_source.hint());
                if !self.env_load_error.is_empty() {
                    ui.colored_label(egui::Color32::LIGHT_RED, &self.env_load_error);
                }
                ui.horizontal(|ui| {
                    ui.label("scope");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.env_import_scope)
                            .hint_text("env")
                            .desired_width(160.0),
                    );
                });
                ui.horizontal(|ui| {
                    ui.label("filter");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.env_import_filter)
                            .hint_text("name prefix")
                            .desired_width(240.0),
                    );
                    if ui.button("select importable").clicked() {
                        for v in &self.env_vars {
                            if v.importable {
                                self.env_selected.insert(v.key.clone());
                            }
                        }
                    }
                    if ui.button("select none").clicked() {
                        self.env_selected.clear();
                    }
                });
                let filter = self.env_import_filter.trim().to_ascii_uppercase();
                let mut rows: Vec<EnvImportRow> = self
                    .env_vars
                    .iter()
                    .filter(|v| filter.is_empty() || v.key.to_ascii_uppercase().contains(&filter))
                    .map(|v| EnvImportRow {
                        key: v.key.clone(),
                        value: v.value.clone(),
                        importable: v.importable,
                        selected: self.env_selected.contains(&v.key),
                    })
                    .collect();
                egui::ScrollArea::vertical()
                    .max_height(280.0)
                    .show(ui, |ui| {
                        for row in &mut rows {
                            ui.horizontal(|ui| {
                                let mut selected = row.selected;
                                if !row.importable {
                                    ui.add_enabled(false, egui::Checkbox::without_text(&mut false));
                                } else if ui.checkbox(&mut selected, "").changed() {
                                    if selected {
                                        self.env_selected.insert(row.key.clone());
                                    } else {
                                        self.env_selected.remove(&row.key);
                                    }
                                }
                                ui.monospace(&row.key);
                                if !row.importable {
                                    ui.colored_label(
                                        egui::Color32::GRAY,
                                        "(name not allowed)",
                                    );
                                } else {
                                    let preview = if system_env::guess_is_secret(&row.key) {
                                        "••••••••".to_string()
                                    } else if row.value.len() > 48 {
                                        format!("{}…", &row.value[..48])
                                    } else {
                                        row.value.clone()
                                    };
                                    ui.label(preview);
                                }
                                if row.importable && ui.small_button("fill form").clicked() {
                                    self.form.key = row.key.clone();
                                    self.form.value = row.value.clone();
                                    self.form.scope = self.env_import_scope.clone();
                                    self.form.is_secret = system_env::guess_is_secret(&row.key);
                                    self.editing_id = None;
                                    self.env_import_open = false;
                                    self.message = format!("filled form: {}", row.key);
                                }
                            });
                        }
                    });
                ui.separator();
                ui.horizontal(|ui| {
                    let n = self.env_selected.len();
                    ui.label(format!("{n} selected"));
                    if ui.button("import selected").clicked() {
                        self.run_env_import();
                    }
                    if ui.button("close").clicked() {
                        self.env_import_open = false;
                    }
                });
            });
        self.env_import_open = open;
    }
}
