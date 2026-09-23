use eframe::egui;

use crate::models::NewVariable;

use super::KeyDesk;
use super::theme;

pub(super) struct SecretLine {
    pub label: &'static str,
    pub value: String,
    pub revealed: bool,
}

pub(super) struct ReplacePrompt {
    pub input: NewVariable,
    pub headline: String,
    pub replacing_stored: bool,
    pub info: Vec<(&'static str, String)>,
    pub secrets: Vec<SecretLine>,
}

impl ReplacePrompt {
    pub(super) fn from_existing(
        input: NewVariable,
        system_value: Option<String>,
        stored: Option<(String, String, String)>,
    ) -> Self {
        let key = input.key.clone();
        let in_system = system_value.is_some();
        let replacing_stored = stored.is_some();
        let mut info = vec![("name", key.clone())];
        let mut secrets = Vec::new();
        let both = system_value.is_some() && stored.is_some();
        if let Some(value) = system_value {
            secrets.push(SecretLine {
                label: if both { "system" } else { "value" },
                value,
                revealed: false,
            });
        }
        if let Some((scope, note, value)) = stored {
            info.push(("scope", scope));
            if !note.is_empty() {
                info.push(("note", note));
            }
            secrets.push(SecretLine {
                label: if both { "stored" } else { "value" },
                value,
                revealed: false,
            });
        }
        let headline = if replacing_stored {
            format!("{key} already exists in this scope. Replace the stored entry?")
        } else if in_system {
            format!("{key} exists in your login shell. Save a separate local entry?")
        } else {
            format!("Save {key}?")
        };
        Self {
            input,
            headline,
            replacing_stored,
            info,
            secrets,
        }
    }
}

impl KeyDesk {
    pub(super) fn show_replace_prompt(&mut self, ctx: &egui::Context) {
        if self.replace_prompt.is_none() {
            return;
        }
        let mut action = None;
        let response = egui::Modal::new(egui::Id::new("replace-existing"))
            .frame(theme::ruled_frame())
            .backdrop_color(egui::Color32::from_black_alpha(160))
            .show(ctx, |ui| {
                ui.set_min_width(380.0);
                ui.set_max_width(460.0);
                ui.monospace("ALREADY EXISTS");
                ui.add_space(6.0);
                let headline = self
                    .replace_prompt
                    .as_ref()
                    .map(|prompt| prompt.headline.clone())
                    .unwrap_or_default();
                ui.add(egui::Label::new(egui::RichText::new(headline).monospace()).wrap());
                ui.add_space(8.0);
                let prompt = self.replace_prompt.as_mut().expect("prompt");
                egui::ScrollArea::vertical()
                    .max_height(180.0)
                    .show(ui, |ui| {
                        egui::Grid::new("replace-info")
                            .num_columns(2)
                            .show(ui, |ui| {
                                for (label, value) in &prompt.info {
                                    ui.monospace(*label);
                                    ui.add_sized(
                                        [300.0, 22.0],
                                        egui::Label::new(egui::RichText::new(value).monospace())
                                            .truncate(),
                                    )
                                    .on_hover_text(value);
                                    ui.end_row();
                                }
                                for secret in &mut prompt.secrets {
                                    ui.monospace(secret.label);
                                    ui.vertical(|ui| {
                                        ui.set_max_width(300.0);
                                        let shown = if secret.revealed {
                                            secret.value.clone()
                                        } else {
                                            "••••".to_string()
                                        };
                                        ui.add(
                                            egui::Label::new(
                                                egui::RichText::new(shown).monospace(),
                                            )
                                            .wrap(),
                                        );
                                        if ui
                                            .small_button(if secret.revealed {
                                                "HIDE"
                                            } else {
                                                "SHOW"
                                            })
                                            .clicked()
                                        {
                                            secret.revealed = !secret.revealed;
                                        }
                                    });
                                    ui.end_row();
                                }
                            });
                    });
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui
                        .button(if prompt.replacing_stored {
                            "REPLACE LOCAL"
                        } else {
                            "SAVE LOCAL"
                        })
                        .clicked()
                    {
                        action = Some(true);
                    }
                    if ui.button("CANCEL").clicked() {
                        action = Some(false);
                    }
                });
            });
        if action.is_none() && response.should_close() {
            action = Some(false);
        }
        match action {
            Some(true) => self.confirm_replace(),
            Some(false) => self.replace_prompt = None,
            None => {}
        }
    }
}
