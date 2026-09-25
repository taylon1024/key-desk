use eframe::egui;

use crate::models::Variable;
use crate::provider::{self, LlmTemplate};

use super::KeyDesk;
use super::theme;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum EntryKind {
    System,
    LlmKey,
}

pub(super) struct Form {
    pub key: String,
    pub value: String,
    pub scope: String,
    pub scope_new: bool,
    pub description: String,
    pub is_secret: bool,
    pub kind: EntryKind,
    pub provider_label: String,
    pub base_url: String,
    pub base_url_name: String,
    pub value_hint: String,
}

impl Default for Form {
    fn default() -> Self {
        Self {
            key: String::new(),
            value: String::new(),
            scope: "default".to_string(),
            scope_new: false,
            description: String::new(),
            is_secret: false,
            kind: EntryKind::System,
            provider_label: String::new(),
            base_url: String::new(),
            base_url_name: String::new(),
            value_hint: String::new(),
        }
    }
}

fn apply_template(form: &mut Form, template: &LlmTemplate) {
    form.provider_label = template.label.to_string();
    form.key = template.api_key_name.to_string();
    form.value.clear();
    form.value_hint = template.placeholder_key.to_string();
    form.base_url = template.base_url.to_string();
    form.base_url_name = template.base_url_name.to_string();
    form.is_secret = true;
    form.description = format!("Provider key: {}", template.label);
}

impl KeyDesk {
    pub(super) fn show_form(&mut self, ui: &mut egui::Ui) {
        // Reserve the provider row so changing entry type does not move the frame.
        ui.set_min_height(250.0);
        theme::ink_bar(
            ui,
            if self.editing_id.is_none() {
                "NEW ENTRY"
            } else {
                "EDIT ENTRY"
            },
            "EDITOR",
        );
        ui.add_space(4.0);
        let creating = self.editing_id.is_none();
        if creating {
            ui.horizontal(|ui| {
                ui.monospace("kind");
                if ui
                    .add_sized(
                        [96.0, 26.0],
                        egui::Button::selectable(self.form.kind == EntryKind::System, "system")
                            .frame_when_inactive(true),
                    )
                    .clicked()
                {
                    self.form.kind = EntryKind::System;
                }
                if ui
                    .add_sized(
                        [128.0, 26.0],
                        egui::Button::selectable(
                            self.form.kind == EntryKind::LlmKey,
                            "provider key",
                        )
                        .frame_when_inactive(true),
                    )
                    .clicked()
                {
                    self.form.kind = EntryKind::LlmKey;
                }
            });
            if self.form.kind == EntryKind::LlmKey {
                let templates = provider::llm_templates();
                let selected = if self.form.provider_label.is_empty() {
                    "provider".to_string()
                } else {
                    self.form.provider_label.clone()
                };
                let mut picked: Option<LlmTemplate> = None;
                ui.horizontal(|ui| {
                    theme::provider_icon(ui, &self.form.provider_label);
                    egui::ComboBox::from_id_salt("llm-provider")
                        .selected_text(selected)
                        .width(180.0)
                        .truncate()
                        .show_ui(ui, |ui| {
                            ui.set_min_width(220.0);
                            for template in templates {
                                ui.horizontal(|ui| {
                                    theme::provider_icon(ui, template.label);
                                    if ui
                                        .selectable_label(
                                            self.form.provider_label == template.label,
                                            template.label,
                                        )
                                        .clicked()
                                    {
                                        picked = Some(*template);
                                    }
                                });
                            }
                        });
                    let key_name = if self.form.key.is_empty() {
                        "select provider"
                    } else {
                        self.form.key.as_str()
                    };
                    ui.add_sized(
                        [220.0, 24.0],
                        egui::Label::new(egui::RichText::new(key_name).monospace()).truncate(),
                    )
                    .on_hover_text(key_name);
                });
                if let Some(template) = picked {
                    apply_template(&mut self.form, &template);
                }
            }
        }
        let llm = creating && self.form.kind == EntryKind::LlmKey;
        const LABEL_WIDTH: f32 = 88.0;
        let field_width =
            (ui.available_width() - LABEL_WIDTH - ui.spacing().item_spacing.x).max(120.0);
        egui::Grid::new("form").num_columns(2).show(ui, |ui| {
            if llm {
                form_label(ui, "base url", LABEL_WIDTH);
                ui.add(
                    egui::TextEdit::singleline(&mut self.form.base_url)
                        .hint_text("https://api.openai.com/v1")
                        .desired_width(field_width)
                        .font(egui::TextStyle::Monospace),
                );
            } else {
                form_label(ui, "name", LABEL_WIDTH);
                ui.add(
                    egui::TextEdit::singleline(&mut self.form.key)
                        .hint_text("DATABASE_URL")
                        .desired_width(field_width)
                        .font(egui::TextStyle::Monospace),
                );
            }
            ui.end_row();

            form_label(ui, "scope", LABEL_WIDTH);
            let creating_scope = self.form.scope_new;
            let scopes = scope_choices(&self.variables, &self.form.scope, creating_scope);
            ui.horizontal(|ui| {
                show_scope_menu(
                    ui,
                    "form-scope",
                    &scopes,
                    &mut self.form.scope,
                    &mut self.form.scope_new,
                    200.0,
                    (field_width - 208.0).min(220.0),
                );
            });
            ui.end_row();

            form_label(ui, if llm { "api key" } else { "value" }, LABEL_WIDTH);
            let mut value = egui::TextEdit::singleline(&mut self.form.value)
                .desired_width(field_width)
                .font(egui::TextStyle::Monospace);
            if llm {
                value = value.hint_text(if self.form.value_hint.is_empty() {
                    "paste API key"
                } else {
                    &self.form.value_hint
                });
            }
            if self.form.is_secret {
                value = value.password(true);
            }
            ui.add(value);
            ui.end_row();

            form_label(ui, "note", LABEL_WIDTH);
            ui.add(
                egui::TextEdit::singleline(&mut self.form.description)
                    .desired_width(field_width)
                    .font(egui::TextStyle::Monospace),
            );
            ui.end_row();
        });

        ui.horizontal(|ui| {
            ui.checkbox(&mut self.form.is_secret, "secret");
            if ui
                .button(if self.editing_id.is_none() {
                    "ADD"
                } else {
                    "SAVE"
                })
                .clicked()
            {
                self.save();
            }
            if self.editing_id.is_some() && ui.button("CANCEL").clicked() {
                self.form = Form::default();
                self.editing_id = None;
            }
        });
    }
}

fn form_label(ui: &mut egui::Ui, text: &str, width: f32) {
    ui.add_sized(
        [width, 26.0],
        egui::Label::new(egui::RichText::new(text).monospace()),
    );
}

pub(super) fn scope_choices(
    variables: &[Variable],
    current: &str,
    creating_new: bool,
) -> Vec<String> {
    let mut scopes: Vec<String> = variables
        .iter()
        .map(|variable| variable.scope.clone())
        .collect();
    if !creating_new && !current.is_empty() {
        scopes.push(current.to_string());
    }
    scopes.push("default".to_string());
    scopes.sort();
    scopes.dedup();
    scopes
}

pub(super) fn show_scope_menu(
    ui: &mut egui::Ui,
    id: &str,
    scopes: &[String],
    scope: &mut String,
    creating_new: &mut bool,
    combo_width: f32,
    new_width: f32,
) {
    let selected = if *creating_new {
        "new".to_string()
    } else {
        scope.clone()
    };
    let mut picked = None;
    let mut pick_new = false;
    let response = egui::ComboBox::from_id_salt(id)
        .selected_text(selected)
        .width(combo_width)
        .truncate()
        .show_ui(ui, |ui| {
            ui.set_min_width(260.0);
            for name in scopes {
                if ui
                    .add_sized(
                        [240.0, 24.0],
                        egui::Button::selectable(!*creating_new && scope == name, name).truncate(),
                    )
                    .on_hover_text(name)
                    .clicked()
                {
                    picked = Some(name.clone());
                }
            }
            ui.separator();
            if ui.selectable_label(*creating_new, "new").clicked() {
                pick_new = true;
            }
        });
    response.response.on_hover_text(scope.as_str());
    if let Some(name) = picked {
        *scope = name;
        *creating_new = false;
    }
    if pick_new && !*creating_new {
        *creating_new = true;
        scope.clear();
    }
    if *creating_new {
        ui.add(
            egui::TextEdit::singleline(scope)
                .hint_text("new scope")
                .desired_width(new_width)
                .font(egui::TextStyle::Monospace),
        );
    }
}

#[cfg(test)]
#[test]
fn form_frame_height_is_stable_across_modes() {
    let mut heights = Vec::new();
    egui::__run_test_ui(|ui| {
        ui.set_width(600.0);
        for (kind, editing, new_scope) in [
            (EntryKind::System, false, false),
            (EntryKind::LlmKey, false, false),
            (EntryKind::System, true, false),
            (EntryKind::LlmKey, false, true),
        ] {
            let mut app = KeyDesk::locked();
            app.form.kind = kind;
            app.form.scope_new = new_scope;
            app.editing_id = editing.then_some(1);
            let frame = theme::ruled_frame(ui.ctx()).show(ui, |ui| app.show_form(ui));
            heights.push(frame.response.rect.height());
        }
    });
    for pass in heights.chunks_exact(4) {
        assert!(pass.iter().all(|height| *height == pass[0]), "{pass:?}");
    }
}

#[cfg(test)]
#[test]
fn provider_template_does_not_fill_a_fake_secret() {
    let mut form = Form {
        value: "an older provider key".to_string(),
        ..Form::default()
    };
    apply_template(&mut form, &provider::llm_templates()[0]);
    assert!(form.value.is_empty());
    assert!(!form.value_hint.is_empty());
}
