use eframe::egui;

use crate::provider::{self, LlmTemplate};

use super::theme;
use super::KeyDesk;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum EntryKind {
    System,
    LlmKey,
}

pub(super) struct Form {
    pub key: String,
    pub value: String,
    pub scope: String,
    pub description: String,
    pub is_secret: bool,
    pub kind: EntryKind,
    pub provider_label: String,
    pub base_url: String,
    pub base_url_name: String,
}

impl Default for Form {
    fn default() -> Self {
        Self {
            key: String::new(),
            value: String::new(),
            scope: "default".to_string(),
            description: String::new(),
            is_secret: false,
            kind: EntryKind::System,
            provider_label: String::new(),
            base_url: String::new(),
            base_url_name: String::new(),
        }
    }
}

fn apply_template(form: &mut Form, template: &LlmTemplate) {
    form.provider_label = template.label.to_string();
    form.key = template.api_key_name.to_string();
    form.value = template.placeholder_key.to_string();
    form.base_url = template.base_url.to_string();
    form.base_url_name = template.base_url_name.to_string();
    form.is_secret = true;
    form.description = format!("LLMKey {}", template.label);
}

impl KeyDesk {
    pub(super) fn show_form(&mut self, ui: &mut egui::Ui) {
        ui.monospace(if self.editing_id.is_none() {
            "NEW ENTRY"
        } else {
            "EDIT ENTRY"
        });
        ui.add_space(4.0);
        let creating = self.editing_id.is_none();
        if creating {
            ui.horizontal(|ui| {
                ui.monospace("kind");
                ui.selectable_value(&mut self.form.kind, EntryKind::System, "system");
                ui.selectable_value(&mut self.form.kind, EntryKind::LlmKey, "LLMKey");
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
                        .show_ui(ui, |ui| {
                            for template in &templates {
                                ui.horizontal(|ui| {
                                    theme::provider_icon(ui, template.label);
                                    if ui
                                        .selectable_label(
                                            self.form.provider_label == template.label,
                                            template.label,
                                        )
                                        .clicked()
                                    {
                                        picked = Some(LlmTemplate {
                                            label: template.label,
                                            api_key_name: template.api_key_name,
                                            base_url_name: template.base_url_name,
                                            base_url: template.base_url,
                                            placeholder_key: template.placeholder_key,
                                        });
                                    }
                                });
                            }
                        });
                });
                if let Some(template) = picked {
                    apply_template(&mut self.form, &template);
                }
            }
        }
        let llm = creating && self.form.kind == EntryKind::LlmKey;
        egui::Grid::new("form").num_columns(2).show(ui, |ui| {
            if llm {
                ui.monospace("base url");
                ui.add(
                    egui::TextEdit::singleline(&mut self.form.base_url)
                        .hint_text("https://api.openai.com/v1")
                        .desired_width(360.0)
                        .font(egui::TextStyle::Monospace),
                );
            } else {
                ui.monospace("name");
                ui.add(
                    egui::TextEdit::singleline(&mut self.form.key)
                        .hint_text("DATABASE_URL")
                        .desired_width(360.0)
                        .font(egui::TextStyle::Monospace),
                );
            }
            ui.end_row();

            ui.monospace("scope");
            ui.add(
                egui::TextEdit::singleline(&mut self.form.scope)
                    .hint_text("default")
                    .desired_width(360.0)
                    .font(egui::TextStyle::Monospace),
            );
            ui.end_row();

            ui.monospace(if llm { "api key" } else { "value" });
            let mut value = egui::TextEdit::singleline(&mut self.form.value)
                .desired_width(360.0)
                .font(egui::TextStyle::Monospace);
            if llm {
                value = value.hint_text("api key");
            }
            if self.form.is_secret {
                value = value.password(true);
            }
            ui.add(value);
            ui.end_row();

            ui.monospace("note");
            ui.add(
                egui::TextEdit::singleline(&mut self.form.description)
                    .desired_width(360.0)
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
