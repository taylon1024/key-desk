//! 窗口界面。按角色拆开：主题、页眉、列表、表单、环境导入。

mod env_import;
mod form;
mod ledger;
mod masthead;
mod replace;
mod theme;

use std::collections::HashSet;

use eframe::egui;
use rusqlite::Connection;

use crate::auth;
use crate::db::{self, DbError};
use crate::models::{self, NewVariable, UpsertVariable, Variable};
use crate::system_env::{EnvSource, EnvVar};

use form::Form;

pub use theme::install_cjk_font;

pub struct KeyDesk {
    db: Option<Connection>,
    auth: Option<auth::PendingAuth>,
    auth_error: String,
    auth_started: bool,
    variables: Vec<Variable>,
    scope_filter: String,
    form: Form,
    editing_id: Option<i64>,
    replace_prompt: Option<replace::ReplacePrompt>,
    revealed: HashSet<i64>,
    message: String,
    env_import_open: bool,
    env_import_source: EnvSource,
    env_import_scope: String,
    env_import_scope_new: bool,
    env_import_filter: String,
    env_vars: Vec<EnvVar>,
    env_selected: HashSet<String>,
    env_load_error: String,
}

impl KeyDesk {
    pub fn locked() -> Self {
        Self {
            db: None,
            auth: None,
            auth_error: String::new(),
            auth_started: false,
            variables: Vec::new(),
            scope_filter: String::new(),
            form: Form::default(),
            editing_id: None,
            replace_prompt: None,
            revealed: HashSet::new(),
            message: String::new(),
            env_import_open: false,
            env_import_source: EnvSource::LoginShell,
            env_import_scope: "env".to_string(),
            env_import_scope_new: false,
            env_import_filter: String::new(),
            env_vars: Vec::new(),
            env_selected: HashSet::new(),
            env_load_error: String::new(),
        }
    }

    fn db(&self) -> &Connection {
        self.db.as_ref().expect("vault is locked")
    }

    fn start_unlock(&mut self, ctx: &egui::Context) {
        if self.auth.is_some() || self.db.is_some() {
            return;
        }
        let ctx = ctx.clone();
        match auth::begin(move || ctx.request_repaint()) {
            Ok(pending) => {
                self.auth_error.clear();
                self.auth = Some(pending);
            }
            Err(err) => self.auth_error = err,
        }
    }

    fn poll_unlock(&mut self, ctx: &egui::Context) {
        if self.db.is_some() {
            return;
        }
        // Development branch opens the vault directly. beta0.0.1 on main still asks for Touch ID or a Mac password.
        #[cfg(feature = "dev")]
        {
            let _ = ctx;
            match db::open(&db::db_path()) {
                Ok(connection) => {
                    self.db = Some(connection);
                    self.reload();
                }
                Err(err) => self.auth_error = format!("failed to open database: {err}"),
            }
        }
        #[cfg(not(feature = "dev"))]
        self.poll_unlock_with_auth(ctx);
    }

    #[cfg(not(feature = "dev"))]
    fn poll_unlock_with_auth(&mut self, ctx: &egui::Context) {
        if self.db.is_some() {
            return;
        }
        if self.auth.is_none() && !self.auth_started {
            self.auth_started = true;
            self.start_unlock(ctx);
        }
        let ready = match self.auth.as_ref() {
            Some(pending) => match pending.rx.try_recv() {
                Ok(result) => Some(result),
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    ctx.request_repaint_after(std::time::Duration::from_millis(100));
                    None
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    Some(Err("authentication stopped".to_string()))
                }
            },
            None => None,
        };
        let Some(result) = ready else {
            return;
        };
        self.auth = None;
        match result {
            Ok(()) => match db::open(&db::db_path()) {
                Ok(connection) => {
                    self.db = Some(connection);
                    self.reload();
                }
                Err(err) => {
                    self.auth_error = format!("failed to open database: {err}");
                    self.auth_started = false;
                }
            },
            Err(err) => self.auth_error = err,
        }
    }

    fn show_lock(&mut self, ui: &mut egui::Ui) {
        theme::ink_bar(ui, "KEYDESK", "LOCKED");
        ui.add_space(8.0);
        theme::ruled_frame().show(ui, |ui| {
            ui.label(egui::RichText::new("LOCKED").monospace().size(28.0));
            ui.monospace(auth::lock_prompt());
            ui.monospace(auth::lock_detail());
            if self.auth.is_some() {
                ui.monospace("waiting...");
            }
            if !self.auth_error.is_empty() {
                ui.monospace(&self.auth_error);
            }
            if self.auth.is_none() && ui.button("UNLOCK").clicked() {
                self.auth_started = true;
                self.start_unlock(ui.ctx());
            }
        });
    }

    fn reload(&mut self) {
        match db::list(self.db(), None) {
            Ok(variables) => {
                self.variables = variables;
                if !self.scope_filter.is_empty()
                    && !self
                        .variables
                        .iter()
                        .any(|variable| variable.scope == self.scope_filter)
                {
                    self.scope_filter.clear();
                }
            }
            Err(err) => self.message = db_message(err),
        }
    }

    fn save(&mut self) {
        if self.editing_id.is_none()
            && self.form.kind == form::EntryKind::LlmKey
            && self.form.provider_label.is_empty()
        {
            self.message = "choose a provider first".to_string();
            return;
        }
        let input = UpsertVariable {
            key: self.form.key.clone(),
            value: self.form.value.clone(),
            scope: Some(self.form.scope.clone()),
            description: Some(self.form.description.clone()),
            is_secret: Some(self.form.is_secret),
        };
        let input = match input.normalize() {
            Ok(input) => input,
            Err(err) => {
                self.message = err;
                return;
            }
        };
        if self.editing_id.is_none() && self.prompt_if_name_taken(&input) {
            return;
        }
        self.persist(input, false);
    }

    fn prompt_if_name_taken(&mut self, input: &NewVariable) -> bool {
        let system_value = match crate::system_env::lookup_login_shell_var(&input.key) {
            Ok(value) => value,
            Err(err) => {
                self.message = err;
                return true;
            }
        };
        let stored = match db::find_in_scope(self.db(), &input.scope, &input.key) {
            Ok(Some(existing)) => Some((existing.scope, existing.description, existing.value)),
            Ok(None) => None,
            Err(err) => {
                self.message = db_message(err);
                return true;
            }
        };
        if system_value.is_none() && stored.is_none() {
            return false;
        }
        self.replace_prompt = Some(replace::ReplacePrompt::from_existing(
            input.clone(),
            system_value,
            stored,
        ));
        true
    }

    fn confirm_replace(&mut self) {
        let Some(prompt) = self.replace_prompt.take() else {
            return;
        };
        self.persist(prompt.input, prompt.replacing_stored);
    }

    fn persist(&mut self, input: NewVariable, replace_existing: bool) {
        let result = if let Some(id) = self.editing_id {
            db::update(self.db(), id, &input)
        } else if replace_existing {
            self.write_new_or_replace(&input)
        } else {
            db::insert(self.db(), &input)
        };
        match result {
            Ok(_) => {
                let mut message = if replace_existing {
                    "replaced".to_string()
                } else {
                    "saved".to_string()
                };
                if self.editing_id.is_none()
                    && self.form.kind == form::EntryKind::LlmKey
                    && !self.form.base_url.is_empty()
                    && !self.form.base_url_name.is_empty()
                {
                    let base = UpsertVariable {
                        key: self.form.base_url_name.clone(),
                        value: self.form.base_url.clone(),
                        scope: Some(input.scope.clone()),
                        description: Some(format!("{} base url", self.form.provider_label)),
                        is_secret: Some(false),
                    };
                    message = match base.normalize().and_then(|row| {
                        if replace_existing {
                            self.write_new_or_replace(&row)
                        } else {
                            db::insert(self.db(), &row)
                        }
                        .map(|_| ())
                        .map_err(db_message)
                    }) {
                        Ok(()) => {
                            if replace_existing {
                                "replaced key and base url".to_string()
                            } else {
                                "saved key and base url".to_string()
                            }
                        }
                        Err(err) => format!("saved key; base url skipped: {err}"),
                    };
                }
                self.message = message;
                self.form = Form::default();
                self.editing_id = None;
                self.reload();
            }
            Err(err) => self.message = db_message(err),
        }
    }

    fn write_new_or_replace(&self, row: &NewVariable) -> Result<Variable, DbError> {
        match db::find_in_scope(self.db(), &row.scope, &row.key) {
            Ok(Some(existing)) => db::update(self.db(), existing.id, row),
            Ok(None) => db::insert(self.db(), row),
            Err(err) => Err(err),
        }
    }

    fn delete(&mut self, id: i64) {
        let deleted_scope = self
            .variables
            .iter()
            .find(|variable| variable.id == id)
            .map(|variable| variable.scope.clone());
        match db::delete(self.db(), id) {
            Ok(()) => {
                if self.editing_id == Some(id) {
                    self.form = Form::default();
                    self.editing_id = None;
                }
                self.revealed.remove(&id);
                self.message = "deleted".to_string();
                self.reload();
                if let Some(scope) = deleted_scope
                    && !self
                        .variables
                        .iter()
                        .any(|variable| variable.scope == scope)
                {
                    if self.form.scope == scope && !self.form.scope_new && self.editing_id.is_none()
                    {
                        self.form.scope = "default".to_string();
                    }
                    if self.env_import_scope == scope && !self.env_import_scope_new {
                        self.env_import_scope = "env".to_string();
                    }
                }
            }
            Err(err) => self.message = db_message(err),
        }
    }

    fn export(&mut self, ctx: &egui::Context) {
        let selected = self
            .variables
            .iter()
            .filter(|variable| self.scope_filter.is_empty() || variable.scope == self.scope_filter)
            .cloned()
            .collect::<Vec<_>>();
        if selected.is_empty() {
            self.message = "nothing to copy".to_string();
            return;
        }
        ctx.copy_text(models::format_dotenv(&selected));
        self.message = "copied to clipboard".to_string();
    }

    fn start_edit(&mut self, variable: &Variable) {
        self.editing_id = Some(variable.id);
        self.form = Form {
            key: variable.key.clone(),
            value: variable.value.clone(),
            scope: variable.scope.clone(),
            description: variable.description.clone(),
            is_secret: variable.is_secret,
            ..Form::default()
        };
    }

    fn show(&mut self, ui: &mut egui::Ui) {
        theme::ink_bar(ui, "KEYDESK", "LOCAL STORE");
        ui.add_space(8.0);
        theme::ruled_frame().show(ui, |ui| self.show_masthead(ui));
        ui.add_space(8.0);
        // Keep the status line in the layout even before the first action.
        // Otherwise every save, copy, or validation error moves both panels.
        ui.add_sized(
            [ui.available_width(), 20.0],
            egui::Label::new(egui::RichText::new(&self.message).monospace()).truncate(),
        )
        .on_hover_text(&self.message);
        ui.add_space(6.0);
        theme::ruled_frame().show(ui, |ui| self.show_list(ui));
        ui.add_space(8.0);
        theme::ruled_frame().show(ui, |ui| self.show_form(ui));
        ui.add_space(8.0);
        theme::dotted_band(ui);
    }
}

impl eframe::App for KeyDesk {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.poll_unlock(ui.ctx());
        egui::Frame::central_panel(ui.style()).show(ui, |ui| {
            if self.db.is_none() {
                self.show_lock(ui);
            } else {
                self.show_env_import_window(ui.ctx());
                self.show(ui);
                self.show_replace_prompt(ui.ctx());
            }
        });
    }
}

fn db_message(err: DbError) -> String {
    match err {
        DbError::NotFound => "variable not found".to_string(),
        DbError::Conflict => "name already exists in this scope".to_string(),
        DbError::Crypto(err) => format!("crypto failed: {err}"),
        DbError::Other(err) => format!("database error: {err}"),
    }
}

#[cfg(test)]
#[test]
fn main_layout_fits_the_minimum_window_height() {
    egui::__run_test_ui(|ui| {
        ui.set_width(640.0);
        let mut app = KeyDesk::locked();
        app.show(ui);
        assert!(
            ui.min_rect().height() <= 724.0,
            "{}",
            ui.min_rect().height()
        );
    });
}
