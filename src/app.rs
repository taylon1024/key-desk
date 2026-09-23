use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

use eframe::egui;
use rusqlite::Connection;

use crate::db::{self, DbError};
use crate::models::{self, UpsertVariable, Variable};
use crate::system_env::{self, EnvSource, EnvVar};

pub struct KeyDesk {
    db: Connection,
    variables: Vec<Variable>,
    scope_filter: String,
    form: Form,
    editing_id: Option<i64>,
    revealed: HashSet<i64>,
    message: String,
    env_import_open: bool,
    env_import_source: EnvSource,
    env_import_scope: String,
    env_import_filter: String,
    env_vars: Vec<EnvVar>,
    env_selected: HashSet<String>,
    env_load_error: String,
}

struct EnvImportRow {
    key: String,
    value: String,
    importable: bool,
    selected: bool,
}

struct Form {
    key: String,
    value: String,
    scope: String,
    description: String,
    is_secret: bool,
}

impl Default for Form {
    fn default() -> Self {
        Self {
            key: String::new(),
            value: String::new(),
            scope: "default".to_string(),
            description: String::new(),
            is_secret: false,
        }
    }
}

impl KeyDesk {
    pub fn new(db: Connection) -> Self {
        let mut app = Self {
            db,
            variables: Vec::new(),
            scope_filter: String::new(),
            form: Form::default(),
            editing_id: None,
            revealed: HashSet::new(),
            message: String::new(),
            env_import_open: false,
            env_import_source: EnvSource::LoginShell,
            env_import_scope: "env".to_string(),
            env_import_filter: String::new(),
            env_vars: Vec::new(),
            env_selected: HashSet::new(),
            env_load_error: String::new(),
        };
        app.reload();
        app
    }

    fn open_env_import(&mut self) {
        self.env_import_filter.clear();
        self.env_import_open = true;
        self.refresh_env_snapshot();
    }

    fn refresh_env_snapshot(&mut self) {
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

    fn run_env_import(&mut self) {
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
            &self.db,
            entries,
            &scope,
            self.env_import_source.import_description(),
        );
        self.reload();
        self.env_import_open = false;
        self.message = format!(
            "环境导入完成：成功 {} 条，校验跳过 {} 条，重名跳过 {} 条",
            report.imported, report.skipped_invalid, report.skipped_conflict
        );
        if let Some(err) = report.errors.first() {
            self.message.push_str("；");
            self.message.push_str(err);
        }
    }

    fn show_env_import_window(&mut self, ctx: &egui::Context) {
        if !self.env_import_open {
            return;
        }
        let mut open = self.env_import_open;
        egui::Window::new("从环境导入")
            .open(&mut open)
            .default_width(520.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("来源");
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
                    if ui.button("刷新").clicked() {
                        self.refresh_env_snapshot();
                    }
                    ui.label(format!("共 {} 条", self.env_vars.len()));
                });
                ui.label(self.env_import_source.hint());
                if !self.env_load_error.is_empty() {
                    ui.colored_label(egui::Color32::LIGHT_RED, &self.env_load_error);
                }
                ui.horizontal(|ui| {
                    ui.label("导入作用域");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.env_import_scope)
                            .hint_text("env")
                            .desired_width(160.0),
                    );
                });
                ui.horizontal(|ui| {
                    ui.label("筛选");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.env_import_filter)
                            .hint_text("变量名前缀")
                            .desired_width(240.0),
                    );
                    if ui.button("全选可导入").clicked() {
                        for v in &self.env_vars {
                            if v.importable {
                                self.env_selected.insert(v.key.clone());
                            }
                        }
                    }
                    if ui.button("全不选").clicked() {
                        self.env_selected.clear();
                    }
                });
                let filter = self.env_import_filter.trim().to_ascii_uppercase();
                let mut rows: Vec<EnvImportRow> = self
                    .env_vars
                    .iter()
                    .filter(|v| {
                        filter.is_empty()
                            || v.key.to_ascii_uppercase().contains(&filter)
                    })
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
                                        "（名称不符合应用规则，已跳过）",
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
                                if row.importable
                                    && ui.small_button("填入表单").clicked()
                                {
                                    self.form.key = row.key.clone();
                                    self.form.value = row.value.clone();
                                    self.form.scope = self.env_import_scope.clone();
                                    self.form.is_secret = system_env::guess_is_secret(&row.key);
                                    self.editing_id = None;
                                    self.env_import_open = false;
                                    self.message =
                                        format!("已填入表单：{}", row.key);
                                }
                            });
                        }
                    });
                ui.separator();
                ui.horizontal(|ui| {
                    let n = self.env_selected.len();
                    ui.label(format!("已选 {n} 条"));
                    if ui.button("导入选中").clicked() {
                        self.run_env_import();
                    }
                    if ui.button("关闭").clicked() {
                        self.env_import_open = false;
                    }
                });
            });
        self.env_import_open = open;
    }

    fn reload(&mut self) {
        match db::list(&self.db, None) {
            Ok(variables) => self.variables = variables,
            Err(err) => self.message = db_message(err),
        }
    }

    fn save(&mut self) {
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
        let result = if let Some(id) = self.editing_id {
            db::update(&self.db, id, &input)
        } else {
            db::insert(&self.db, &input)
        };
        match result {
            Ok(_) => {
                self.message = "已保存".to_string();
                self.form = Form::default();
                self.editing_id = None;
                self.reload();
            }
            Err(err) => self.message = db_message(err),
        }
    }

    fn delete(&mut self, id: i64) {
        match db::delete(&self.db, id) {
            Ok(()) => {
                if self.editing_id == Some(id) {
                    self.form = Form::default();
                    self.editing_id = None;
                }
                self.revealed.remove(&id);
                self.message = "已删除".to_string();
                self.reload();
            }
            Err(err) => self.message = db_message(err),
        }
    }

    fn export(&mut self) {
        let selected = self
            .variables
            .iter()
            .filter(|variable| self.scope_filter.is_empty() || variable.scope == self.scope_filter)
            .cloned()
            .collect::<Vec<_>>();
        let text = models::format_dotenv(&selected);
        let name = if self.scope_filter.is_empty() {
            "all.env".to_string()
        } else {
            format!("{}.env", self.scope_filter)
        };
        let path = export_dir().join(name);
        match std::fs::write(&path, text) {
            Ok(()) => self.message = format!("已导出 {}", path.display()),
            Err(err) => self.message = format!("导出失败: {err}"),
        }
    }

    fn start_edit(&mut self, variable: &Variable) {
        self.editing_id = Some(variable.id);
        self.form = Form {
            key: variable.key.clone(),
            value: variable.value.clone(),
            scope: variable.scope.clone(),
            description: variable.description.clone(),
            is_secret: variable.is_secret,
        };
    }

    fn show(&mut self, ui: &mut egui::Ui) {
        ui.heading("key-desk");
        if !self.message.is_empty() {
            ui.label(&self.message);
        }

        ui.separator();
        self.show_form(ui);
        ui.separator();
        self.show_list(ui);
    }

    fn show_form(&mut self, ui: &mut egui::Ui) {
        ui.heading(if self.editing_id.is_none() {
            "新增变量"
        } else {
            "编辑变量"
        });

        egui::Grid::new("form").num_columns(2).show(ui, |ui| {
            ui.label("变量名");
            ui.add(
                egui::TextEdit::singleline(&mut self.form.key)
                    .hint_text("DATABASE_URL")
                    .desired_width(320.0),
            );
            ui.end_row();

            ui.label("作用域");
            ui.add(
                egui::TextEdit::singleline(&mut self.form.scope)
                    .hint_text("default")
                    .desired_width(320.0),
            );
            ui.end_row();

            ui.label("值");
            let mut value = egui::TextEdit::singleline(&mut self.form.value).desired_width(320.0);
            if self.form.is_secret {
                value = value.password(true);
            }
            ui.add(value);
            ui.end_row();

            ui.label("说明");
            ui.add(egui::TextEdit::singleline(&mut self.form.description).desired_width(320.0));
            ui.end_row();
        });

        ui.horizontal(|ui| {
            ui.checkbox(&mut self.form.is_secret, "敏感值");
            if ui
                .button(if self.editing_id.is_none() {
                    "添加"
                } else {
                    "保存"
                })
                .clicked()
            {
                self.save();
            }
            if self.editing_id.is_some() && ui.button("取消").clicked() {
                self.form = Form::default();
                self.editing_id = None;
            }
        });
    }

    fn show_list(&mut self, ui: &mut egui::Ui) {
        let mut scopes: Vec<String> = self
            .variables
            .iter()
            .map(|variable| variable.scope.clone())
            .collect();
        scopes.sort();
        scopes.dedup();

        ui.horizontal(|ui| {
            ui.label("作用域");
            egui::ComboBox::from_id_salt("scope-filter")
                .selected_text(if self.scope_filter.is_empty() {
                    "全部"
                } else {
                    self.scope_filter.as_str()
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.scope_filter, String::new(), "全部");
                    for scope in &scopes {
                        ui.selectable_value(&mut self.scope_filter, scope.clone(), scope);
                    }
                });
            if ui.button("导出 .env").clicked() {
                self.export();
            }
            if ui.button("从环境导入…").clicked() {
                self.open_env_import();
            }
            let count = self
                .variables
                .iter()
                .filter(|variable| {
                    self.scope_filter.is_empty() || variable.scope == self.scope_filter
                })
                .count();
            ui.label(format!("{count} 条"));
        });

        let visible: Vec<Variable> = self
            .variables
            .iter()
            .filter(|variable| self.scope_filter.is_empty() || variable.scope == self.scope_filter)
            .cloned()
            .collect();

        egui::ScrollArea::vertical().show(ui, |ui| {
            for variable in visible {
                let shown = !variable.is_secret || self.revealed.contains(&variable.id);
                ui.horizontal(|ui| {
                    ui.monospace(&variable.key);
                    ui.label(&variable.scope);
                    if variable.is_secret {
                        ui.label("敏感");
                    }
                });
                ui.label(if shown {
                    if variable.value.is_empty() {
                        "（空）".to_string()
                    } else {
                        variable.value.clone()
                    }
                } else {
                    "••••••••".to_string()
                });
                if !variable.description.is_empty() {
                    ui.label(&variable.description);
                }
                ui.horizontal(|ui| {
                    if variable.is_secret
                        && ui.button(if shown { "隐藏" } else { "显示" }).clicked()
                    {
                        if shown {
                            self.revealed.remove(&variable.id);
                        } else {
                            self.revealed.insert(variable.id);
                        }
                    }
                    if ui.button("复制").clicked() {
                        ui.ctx().copy_text(variable.value.clone());
                        self.message = format!("已复制 {}", variable.key);
                    }
                    if ui.button("编辑").clicked() {
                        self.start_edit(&variable);
                    }
                    if ui.button("删除").clicked() {
                        self.delete(variable.id);
                    }
                });
                ui.separator();
            }
        });
    }
}

impl eframe::App for KeyDesk {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.show_env_import_window(ui.ctx());
        egui::Frame::central_panel(&ui.style()).show(ui, |ui| self.show(ui));
    }
}

pub fn install_cjk_font(ctx: &egui::Context) {
    let Some(bytes) = cjk_font_bytes() else {
        return;
    };
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "cjk".to_owned(),
        Arc::new(egui::FontData::from_owned(bytes)),
    );
    if let Some(proportional) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
        proportional.insert(0, "cjk".to_owned());
    }
    if let Some(monospace) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
        monospace.push("cjk".to_owned());
    }
    ctx.set_fonts(fonts);
}

fn cjk_font_bytes() -> Option<Vec<u8>> {
    [
        "/System/Library/Fonts/Hiragino Sans GB.ttc",
        "/System/Library/Fonts/STHeiti Medium.ttc",
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
        r"C:\Windows\Fonts\msyh.ttc",
    ]
    .into_iter()
    .find_map(|path| std::fs::read(path).ok())
}

fn export_dir() -> PathBuf {
    db::db_path()
        .parent()
        .map(PathBuf::from)
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| PathBuf::from("data"))
}

fn db_message(err: DbError) -> String {
    match err {
        DbError::NotFound => "变量不存在".to_string(),
        DbError::Conflict => "同一作用域下变量名已存在".to_string(),
        DbError::Other(err) => format!("数据库错误: {err}"),
    }
}
