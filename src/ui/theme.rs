//! 终端风格：字体、线框、反色标签、刻度，以及可切换配色。

use std::path::{Path, PathBuf};
use std::sync::Arc;

use eframe::egui;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ThemeId {
    InkPaper,
    CrtGreen,
    Amber,
    Blueprint,
    Cyber,
    WarmPaper,
    Graphite,
    Sakura,
    Forest,
    Sunset,
    Ice,
    Grape,
}

impl ThemeId {
    pub(super) const DEFAULT: Self = Self::InkPaper;

    pub(super) fn slug(self) -> &'static str {
        self.palette().slug
    }

    pub(super) fn from_slug(slug: &str) -> Option<Self> {
        themes()
            .into_iter()
            .find(|theme| theme.slug == slug)
            .map(|theme| theme.id)
    }

    pub(super) fn palette(self) -> Theme {
        themes()
            .into_iter()
            .find(|theme| theme.id == self)
            .expect("theme palette")
    }
}

/// 一套界面配色。字段对应原来写死的黑白终端色。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Theme {
    pub id: ThemeId,
    pub slug: &'static str,
    pub name: &'static str,
    pub bg: egui::Color32,
    pub faint: egui::Color32,
    pub hover: egui::Color32,
    pub ink: egui::Color32,
    pub stroke: egui::Color32,
    pub inv: egui::Color32,
    pub accent: egui::Color32,
}

impl Theme {
    /// 背景偏暗时用 egui 的 dark visuals 打底，否则用 light。
    pub(super) fn is_dark(self) -> bool {
        let [r, g, b, _] = self.bg.to_array();
        let luminance = 0.2126 * f32::from(r) + 0.7152 * f32::from(g) + 0.0722 * f32::from(b);
        luminance < 128.0
    }
}

fn rgb(hex: u32) -> egui::Color32 {
    egui::Color32::from_rgb(
        ((hex >> 16) & 0xff) as u8,
        ((hex >> 8) & 0xff) as u8,
        (hex & 0xff) as u8,
    )
}

fn theme(
    id: ThemeId,
    slug: &'static str,
    name: &'static str,
    bg: u32,
    faint: u32,
    hover: u32,
    ink: u32,
    stroke: u32,
    inv: u32,
    accent: u32,
) -> Theme {
    Theme {
        id,
        slug,
        name,
        bg: rgb(bg),
        faint: rgb(faint),
        hover: rgb(hover),
        ink: rgb(ink),
        stroke: rgb(stroke),
        inv: rgb(inv),
        accent: rgb(accent),
    }
}

pub(super) fn themes() -> [Theme; 12] {
    use ThemeId::*;
    [
        theme(
            InkPaper,
            "ink_paper",
            "墨纸终端",
            0xFFFFFF,
            0xF5F5F5,
            0xE6E6E6,
            0x111111,
            0x000000,
            0xFFFFFF,
            0x111111,
        ),
        theme(
            CrtGreen,
            "crt_green",
            "CRT 绿屏",
            0x0A1A0A,
            0x102410,
            0x1A3A1A,
            0x33FF66,
            0x22CC55,
            0x0A1A0A,
            0x66FF99,
        ),
        theme(
            Amber,
            "amber",
            "琥珀 Amber",
            0x1A1208,
            0x241808,
            0x3A2810,
            0xFFB000,
            0xE09000,
            0x1A1208,
            0xFFD060,
        ),
        theme(
            Blueprint,
            "blueprint",
            "蓝纸 Blueprint",
            0x0B1F3A,
            0x123052,
            0x1A4068,
            0xE8F0FF,
            0x7EB6FF,
            0x0B1F3A,
            0xFFD166,
        ),
        theme(
            Cyber,
            "cyber",
            "赛博 Cyber",
            0x0D0D12,
            0x16161F,
            0x222233,
            0xE6E6F0,
            0x00F0FF,
            0x0D0D12,
            0xFF2D95,
        ),
        theme(
            WarmPaper,
            "warm_paper",
            "纸感暖白",
            0xF7F1E8,
            0xEFE6D8,
            0xE2D5C2,
            0x2C241B,
            0x5C4A3A,
            0xF7F1E8,
            0xC45C26,
        ),
        theme(
            Graphite,
            "graphite",
            "石墨 Graphite",
            0x1C1C1E,
            0x2A2A2E,
            0x3A3A40,
            0xF2F2F2,
            0x8E8E93,
            0x1C1C1E,
            0x64D2FF,
        ),
        theme(
            Sakura,
            "sakura",
            "樱花 Sakura",
            0xFFF5F7,
            0xFFE8EE,
            0xFFD6E0,
            0x3D2A32,
            0xE89AAB,
            0xFFF5F7,
            0xE85A7A,
        ),
        theme(
            Forest,
            "forest",
            "森林 Forest",
            0xF2F6F1,
            0xE4EDE2,
            0xD0DFCC,
            0x1B2E1F,
            0x2F5D3A,
            0xF2F6F1,
            0xC4A35A,
        ),
        theme(
            Sunset,
            "sunset",
            "日落 Sunset",
            0x1A0F14,
            0x2A1520,
            0x3D1F2E,
            0xFFE8D6,
            0xFF6B4A,
            0x1A0F14,
            0xFFB347,
        ),
        theme(
            Ice,
            "ice",
            "冰蓝 Ice",
            0xF4F8FC,
            0xE6EEF6,
            0xD0DCEB,
            0x0F1C2E,
            0x2B4C7E,
            0xF4F8FC,
            0xE85D04,
        ),
        theme(
            Grape,
            "grape",
            "葡萄 Purple",
            0x16121F,
            0x221A2E,
            0x322640,
            0xF0E6FF,
            0xA78BFA,
            0x16121F,
            0x34D399,
        ),
    ]
}

fn theme_key() -> egui::Id {
    egui::Id::new("keydesk-theme")
}

pub fn install_cjk_font(ctx: &egui::Context) {
    apply_theme(ctx, load_theme_id());
    let mut fonts = egui::FontDefinitions::default();
    let mut family = Vec::new();
    if let Some(bytes) = terminal_font_bytes() {
        fonts.font_data.insert(
            "terminal".to_owned(),
            Arc::new(egui::FontData::from_owned(bytes)),
        );
        family.push("terminal".to_owned());
    }
    if let Some(bytes) = cjk_font_bytes() {
        fonts.font_data.insert(
            "cjk".to_owned(),
            Arc::new(egui::FontData::from_owned(bytes)),
        );
        family.push("cjk".to_owned());
    }
    if family.is_empty() {
        return;
    }
    fonts
        .families
        .insert(egui::FontFamily::Monospace, family.clone());
    fonts
        .families
        .insert(egui::FontFamily::Proportional, family);
    ctx.set_fonts(fonts);
}

fn terminal_font_bytes() -> Option<Vec<u8>> {
    [
        "/System/Library/Fonts/Monaco.ttf",
        "/System/Library/Fonts/Menlo.ttc",
        "/System/Library/Fonts/Supplemental/PTMono.ttc",
        "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf",
        r"C:\Windows\Fonts\consola.ttf",
    ]
    .into_iter()
    .find_map(|path| std::fs::read(path).ok())
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

pub(super) fn apply_theme(ctx: &egui::Context, id: ThemeId) {
    let theme = id.palette();
    let mut visuals = if theme.is_dark() {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };
    visuals.window_fill = theme.bg;
    visuals.panel_fill = theme.bg;
    visuals.extreme_bg_color = theme.bg;
    visuals.faint_bg_color = theme.faint;
    visuals.code_bg_color = theme.bg;
    visuals.hyperlink_color = theme.accent;
    visuals.text_cursor.stroke.color = theme.ink;
    visuals.window_stroke = egui::Stroke::new(2.0, theme.stroke);
    visuals.window_corner_radius = egui::CornerRadius::ZERO;
    visuals.menu_corner_radius = egui::CornerRadius::ZERO;
    visuals.window_shadow = egui::Shadow::NONE;
    visuals.popup_shadow = egui::Shadow::NONE;
    visuals.selection.bg_fill = theme.ink;
    visuals.selection.stroke = egui::Stroke::new(1.0, theme.inv);
    let stroke = egui::Stroke::new(1.5, theme.stroke);
    for widget in [
        &mut visuals.widgets.noninteractive,
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
        &mut visuals.widgets.open,
    ] {
        widget.corner_radius = egui::CornerRadius::ZERO;
        widget.bg_stroke = stroke;
        widget.fg_stroke = egui::Stroke::new(1.0, theme.ink);
        widget.bg_fill = theme.bg;
        widget.weak_bg_fill = theme.bg;
    }
    visuals.widgets.hovered.bg_fill = theme.hover;
    visuals.widgets.hovered.weak_bg_fill = theme.hover;
    visuals.widgets.active.bg_fill = theme.ink;
    visuals.widgets.active.weak_bg_fill = theme.ink;
    visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, theme.inv);

    ctx.set_theme(if theme.is_dark() {
        egui::Theme::Dark
    } else {
        egui::Theme::Light
    });
    ctx.all_styles_mut(|style| {
        style.visuals = visuals.clone();
        style.spacing.item_spacing = egui::vec2(8.0, 6.0);
        style.spacing.button_padding = egui::vec2(8.0, 3.0);
        for (text_style, size) in [
            (egui::TextStyle::Small, 11.0),
            (egui::TextStyle::Body, 14.0),
            (egui::TextStyle::Button, 13.0),
            (egui::TextStyle::Heading, 22.0),
            (egui::TextStyle::Monospace, 14.0),
        ] {
            style.text_styles.insert(
                text_style,
                egui::FontId::new(size, egui::FontFamily::Monospace),
            );
        }
    });
    ctx.data_mut(|data| data.insert_temp(theme_key(), theme));
}

/// 每帧把已保存的主题铺到当前 `Ui`，这样下拉框切换后下一帧就能生效。
pub(super) fn sync_ui(ui: &mut egui::Ui, id: ThemeId) {
    apply_theme(ui.ctx(), id);
    let style = ui.ctx().global_style();
    ui.set_style(style);
}

fn active(ui: &egui::Ui) -> Theme {
    ui.ctx()
        .data(|data| data.get_temp(theme_key()))
        .unwrap_or_else(|| ThemeId::DEFAULT.palette())
}

pub(super) fn theme_picker(ui: &mut egui::Ui, current: &mut ThemeId) -> bool {
    let before = *current;
    egui::ComboBox::from_id_salt("keydesk-theme")
        .selected_text(current.palette().name)
        .width(180.0)
        .show_ui(ui, |ui| {
            for theme in themes() {
                ui.selectable_value(current, theme.id, theme.name);
            }
        });
    *current != before
}

fn prefs_path() -> PathBuf {
    let db = crate::db::db_path();
    match db.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.join("theme_id"),
        _ => PathBuf::from("theme_id"),
    }
}

pub(super) fn load_theme_id() -> ThemeId {
    read_theme_id(&prefs_path())
}

pub(super) fn save_theme_id(id: ThemeId) -> Result<(), String> {
    write_theme_id(&prefs_path(), id)
}

fn read_theme_id(path: &Path) -> ThemeId {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|text| ThemeId::from_slug(text.trim()))
        .unwrap_or(ThemeId::DEFAULT)
}

fn write_theme_id(path: &Path, id: ThemeId) -> Result<(), String> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    std::fs::write(path, format!("{}\n", id.slug())).map_err(|err| err.to_string())
}

pub(super) fn ruled_frame(ui: &egui::Ui) -> egui::Frame {
    let theme = active(ui);
    egui::Frame::new()
        .fill(theme.bg)
        .stroke(egui::Stroke::new(2.0, theme.stroke))
        .inner_margin(10)
        .corner_radius(egui::CornerRadius::ZERO)
}

pub(super) fn ink_bar(ui: &mut egui::Ui, left: &str, right: &str) {
    let theme = active(ui);
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 22.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, 0.0, theme.ink);
    ui.painter().text(
        rect.left_center() + egui::vec2(8.0, 0.0),
        egui::Align2::LEFT_CENTER,
        left,
        egui::FontId::monospace(13.0),
        theme.inv,
    );
    ui.painter().text(
        rect.right_center() + egui::vec2(-8.0, 0.0),
        egui::Align2::RIGHT_CENTER,
        right,
        egui::FontId::monospace(12.0),
        theme.inv,
    );
}

pub(super) fn ink_badge(ui: &mut egui::Ui, text: &str) {
    let theme = active(ui);
    let galley =
        ui.painter()
            .layout_no_wrap(text.to_owned(), egui::FontId::monospace(13.0), theme.inv);
    let pad = egui::vec2(6.0, 2.0);
    let (rect, _) = ui.allocate_exact_size(galley.size() + pad * 2.0, egui::Sense::hover());
    ui.painter().rect_filled(rect, 0.0, theme.ink);
    ui.painter().galley(rect.min + pad, galley, theme.inv);
}

pub(super) fn scale_tick(ui: &mut egui::Ui, t: f32) {
    let theme = active(ui);
    let width = (ui.available_width() - 120.0).max(48.0);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 14.0), egui::Sense::hover());
    let y = rect.center().y;
    ui.painter()
        .hline(rect.x_range(), y, egui::Stroke::new(1.0, theme.stroke));
    let x = rect.left() + rect.width() * t.clamp(0.04, 0.96);
    let mark = egui::Rect::from_center_size(egui::pos2(x, y), egui::vec2(7.0, 7.0));
    ui.painter().rect_filled(mark, 0.0, theme.ink);
}

pub(super) fn tick_at(key: &str) -> f32 {
    let hash = key.bytes().fold(0u32, |acc, byte| {
        acc.wrapping_mul(33).wrapping_add(byte as u32)
    });
    (hash % 100) as f32 / 100.0
}

pub(super) fn hairline(ui: &mut egui::Ui) {
    let theme = active(ui);
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 8.0), egui::Sense::hover());
    ui.painter().hline(
        rect.x_range(),
        rect.center().y,
        egui::Stroke::new(1.0, theme.stroke),
    );
}

pub(super) fn dotted_band(ui: &mut egui::Ui) {
    let theme = active(ui);
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 22.0), egui::Sense::hover());
    ui.painter().rect_stroke(
        rect,
        0.0,
        egui::Stroke::new(2.0, theme.stroke),
        egui::StrokeKind::Inside,
    );
    let mut x = rect.left() + 8.0;
    let y = rect.center().y;
    while x < rect.right() - 6.0 {
        ui.painter().circle_filled(egui::pos2(x, y), 1.1, theme.ink);
        x += 6.0;
    }
}

pub(super) fn provider_icon(ui: &mut egui::Ui, label: &str) {
    let Some(name) = provider_icon_name(label) else {
        let theme = active(ui);
        let (rect, _) = ui.allocate_exact_size(egui::vec2(16.0, 16.0), egui::Sense::hover());
        ui.painter().rect_stroke(
            rect,
            0.0,
            egui::Stroke::new(1.0, theme.stroke),
            egui::StrokeKind::Inside,
        );
        return;
    };
    let bytes = crate::icons::icon_pixel_png(name).expect("provider icon");
    let texture = provider_texture(ui.ctx(), name, bytes);
    ui.add(
        egui::Image::from_texture(&texture)
            .fit_to_exact_size(egui::vec2(16.0, 16.0))
            .texture_options(egui::TextureOptions::NEAREST),
    );
}

fn provider_icon_name(label: &str) -> Option<&'static str> {
    Some(match label {
        "Anthropic" => "antropic",
        "Baichuan" => "baichuan",
        "DashScope" => "dashscope",
        "DeepSeek" => "deepseek",
        "Fireworks" => "fireworks",
        "Gemini" => "gemini",
        "Groq" => "groq",
        "Hunyuan" => "hunyuan",
        "Lingyi" => "lingyi",
        "MiniMax" => "minimax",
        "Mistral" => "mistral",
        "ModelScope" => "modelscope",
        "Moonshot" => "moonshot",
        "OpenAI" => "openai",
        "OpenRouter" => "openrouter",
        "Qianfan" => "qianfan",
        "SiliconFlow" => "siliconflow",
        "StepFun" => "stepfun",
        "Together" => "together",
        "TypeSafeAI" => "typesafeai",
        "Volcengine" => "volcengine",
        "xAI" => "xai",
        "Zhipu" => "zhipu",
        _ => return None,
    })
}

#[derive(Clone, Default)]
struct IconCache {
    textures: std::collections::HashMap<String, egui::TextureHandle>,
}

fn provider_texture(ctx: &egui::Context, name: &str, bytes: &[u8]) -> egui::TextureHandle {
    let id = egui::Id::new("provider-icons");
    if let Some(texture) = ctx.data(|data| {
        data.get_temp::<IconCache>(id)
            .and_then(|cache| cache.textures.get(name).cloned())
    }) {
        return texture;
    }
    let image = decode_png(bytes);
    let texture = ctx.load_texture(
        format!("provider-{name}"),
        image,
        egui::TextureOptions::NEAREST,
    );
    ctx.data_mut(|data| {
        data.get_temp_mut_or_default::<IconCache>(id)
            .textures
            .insert(name.to_string(), texture.clone());
    });
    texture
}

fn decode_png(bytes: &[u8]) -> egui::ColorImage {
    let image = image::load_from_memory(bytes)
        .expect("provider png")
        .to_rgba8();
    egui::ColorImage::from_rgba_unmultiplied(
        [image.width() as usize, image.height() as usize],
        image.as_raw(),
    )
}

pub(super) fn pixel_mark(ui: &mut egui::Ui, size: f32) {
    let theme = active(ui);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());
    ui.painter().rect_stroke(
        rect,
        0.0,
        egui::Stroke::new(2.0, theme.stroke),
        egui::StrokeKind::Inside,
    );
    const CELLS: &[&str] = &[
        "#.#.#.#", "#.....#", "#.###.#", "#.#.#.#", "#.###.#", "#.....#", "#.#.#.#",
    ];
    let cell = (size - 10.0) / CELLS.len() as f32;
    let origin = rect.min + egui::vec2(5.0, 5.0);
    for (row, line) in CELLS.iter().enumerate() {
        for (col, ch) in line.chars().enumerate() {
            if ch == '#' {
                let min = origin + egui::vec2(col as f32 * cell, row as f32 * cell);
                ui.painter().rect_filled(
                    egui::Rect::from_min_size(min, egui::vec2(cell - 0.5, cell - 0.5)),
                    0.0,
                    theme.ink,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn expect(slug: &str, name: &str, dark: bool, colors: [u32; 7]) -> (Theme, bool) {
        let theme = ThemeId::from_slug(slug).expect(slug).palette();
        assert_eq!(theme.slug, slug);
        assert_eq!(theme.name, name);
        assert_eq!(theme.is_dark(), dark);
        assert_eq!(
            [
                theme.bg,
                theme.faint,
                theme.hover,
                theme.ink,
                theme.stroke,
                theme.inv,
                theme.accent
            ],
            colors.map(rgb)
        );
        (theme, dark)
    }

    #[test]
    fn twelve_named_palettes_match_hex() {
        assert_eq!(themes().len(), 12);
        assert_eq!(ThemeId::DEFAULT, ThemeId::InkPaper);
        assert_eq!(ThemeId::DEFAULT.slug(), "ink_paper");
        let cases = [
            (
                "ink_paper",
                "墨纸终端",
                false,
                [
                    0xFFFFFF, 0xF5F5F5, 0xE6E6E6, 0x111111, 0x000000, 0xFFFFFF, 0x111111,
                ],
            ),
            (
                "crt_green",
                "CRT 绿屏",
                true,
                [
                    0x0A1A0A, 0x102410, 0x1A3A1A, 0x33FF66, 0x22CC55, 0x0A1A0A, 0x66FF99,
                ],
            ),
            (
                "amber",
                "琥珀 Amber",
                true,
                [
                    0x1A1208, 0x241808, 0x3A2810, 0xFFB000, 0xE09000, 0x1A1208, 0xFFD060,
                ],
            ),
            (
                "blueprint",
                "蓝纸 Blueprint",
                true,
                [
                    0x0B1F3A, 0x123052, 0x1A4068, 0xE8F0FF, 0x7EB6FF, 0x0B1F3A, 0xFFD166,
                ],
            ),
            (
                "cyber",
                "赛博 Cyber",
                true,
                [
                    0x0D0D12, 0x16161F, 0x222233, 0xE6E6F0, 0x00F0FF, 0x0D0D12, 0xFF2D95,
                ],
            ),
            (
                "warm_paper",
                "纸感暖白",
                false,
                [
                    0xF7F1E8, 0xEFE6D8, 0xE2D5C2, 0x2C241B, 0x5C4A3A, 0xF7F1E8, 0xC45C26,
                ],
            ),
            (
                "graphite",
                "石墨 Graphite",
                true,
                [
                    0x1C1C1E, 0x2A2A2E, 0x3A3A40, 0xF2F2F2, 0x8E8E93, 0x1C1C1E, 0x64D2FF,
                ],
            ),
            (
                "sakura",
                "樱花 Sakura",
                false,
                [
                    0xFFF5F7, 0xFFE8EE, 0xFFD6E0, 0x3D2A32, 0xE89AAB, 0xFFF5F7, 0xE85A7A,
                ],
            ),
            (
                "forest",
                "森林 Forest",
                false,
                [
                    0xF2F6F1, 0xE4EDE2, 0xD0DFCC, 0x1B2E1F, 0x2F5D3A, 0xF2F6F1, 0xC4A35A,
                ],
            ),
            (
                "sunset",
                "日落 Sunset",
                true,
                [
                    0x1A0F14, 0x2A1520, 0x3D1F2E, 0xFFE8D6, 0xFF6B4A, 0x1A0F14, 0xFFB347,
                ],
            ),
            (
                "ice",
                "冰蓝 Ice",
                false,
                [
                    0xF4F8FC, 0xE6EEF6, 0xD0DCEB, 0x0F1C2E, 0x2B4C7E, 0xF4F8FC, 0xE85D04,
                ],
            ),
            (
                "grape",
                "葡萄 Purple",
                true,
                [
                    0x16121F, 0x221A2E, 0x322640, 0xF0E6FF, 0xA78BFA, 0x16121F, 0x34D399,
                ],
            ),
        ];
        assert_eq!(cases.len(), themes().len());
        let mut slugs = std::collections::HashSet::new();
        for (slug, name, dark, colors) in cases {
            expect(slug, name, dark, colors);
            assert!(slugs.insert(slug));
        }
        assert!(ThemeId::from_slug("nope").is_none());
        assert!(ThemeId::from_slug("Ink_Paper").is_none());
    }

    #[test]
    fn theme_id_file_roundtrip() {
        let dir = std::env::temp_dir().join(format!(
            "key-desk-theme-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("theme_id");
        assert_eq!(read_theme_id(&path), ThemeId::InkPaper);
        write_theme_id(&path, ThemeId::Grape).unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap().trim(), "grape");
        assert_eq!(read_theme_id(&path), ThemeId::Grape);
        std::fs::write(&path, "  crt_green \n").unwrap();
        assert_eq!(read_theme_id(&path), ThemeId::CrtGreen);
        std::fs::write(&path, "not-a-theme").unwrap();
        assert_eq!(read_theme_id(&path), ThemeId::InkPaper);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn prefs_live_beside_the_database() {
        let path = prefs_path();
        assert_eq!(path.file_name().unwrap(), "theme_id");
        assert_eq!(path.parent(), crate::db::db_path().parent());
    }

    #[test]
    fn apply_theme_sets_visuals_and_chrome() {
        let ctx = egui::Context::default();
        apply_theme(&ctx, ThemeId::Cyber);
        let theme = ThemeId::Cyber.palette();
        let visuals = ctx.global_style().visuals.clone();
        assert_eq!(ctx.theme(), egui::Theme::Dark);
        assert!(visuals.dark_mode);
        assert_eq!(visuals.window_fill, theme.bg);
        assert_eq!(visuals.panel_fill, theme.bg);
        assert_eq!(visuals.extreme_bg_color, theme.bg);
        assert_eq!(visuals.code_bg_color, theme.bg);
        assert_eq!(visuals.faint_bg_color, theme.faint);
        assert_eq!(visuals.hyperlink_color, theme.accent);
        assert_eq!(visuals.window_stroke.color, theme.stroke);
        assert_eq!(visuals.window_corner_radius, egui::CornerRadius::ZERO);
        assert_eq!(visuals.selection.bg_fill, theme.ink);
        assert_eq!(visuals.selection.stroke.color, theme.inv);
        assert_eq!(visuals.widgets.hovered.bg_fill, theme.hover);
        assert_eq!(visuals.widgets.noninteractive.bg_fill, theme.bg);
        assert_eq!(visuals.widgets.inactive.bg_fill, theme.bg);
        assert_eq!(visuals.widgets.open.bg_fill, theme.bg);
        assert_eq!(visuals.widgets.noninteractive.fg_stroke.color, theme.ink);
        assert_eq!(visuals.widgets.inactive.bg_stroke.color, theme.stroke);
        assert_eq!(visuals.widgets.open.bg_stroke.color, theme.stroke);
        assert_eq!(visuals.widgets.active.bg_fill, theme.ink);
        assert_eq!(visuals.widgets.active.fg_stroke.color, theme.inv);
        assert_eq!(
            visuals.widgets.inactive.corner_radius,
            egui::CornerRadius::ZERO
        );
        assert_eq!(
            ctx.data(|data| data.get_temp::<Theme>(theme_key())),
            Some(theme)
        );

        apply_theme(&ctx, ThemeId::InkPaper);
        let paper = ThemeId::InkPaper.palette();
        let visuals = ctx.global_style().visuals.clone();
        assert_eq!(ctx.theme(), egui::Theme::Light);
        assert!(!visuals.dark_mode);
        assert_eq!(visuals.panel_fill, paper.bg);
        assert_eq!(visuals.widgets.noninteractive.fg_stroke.color, paper.ink);

        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            sync_ui(ui, ThemeId::Sunset);
            let sunset = ThemeId::Sunset.palette();
            assert_eq!(ui.visuals().window_fill, sunset.bg);
            assert!(ui.visuals().dark_mode);
            let frame = ruled_frame(ui);
            assert_eq!(frame.fill, sunset.bg);
            assert_eq!(frame.stroke.color, sunset.stroke);
            assert_eq!(frame.corner_radius, egui::CornerRadius::ZERO);
            ink_bar(ui, "KEYDESK", "LOCAL STORE");
            ink_badge(ui, "AES");
            scale_tick(ui, 0.5);
            hairline(ui);
            dotted_band(ui);
            pixel_mark(ui, 52.0);
        });
        output.textures_delta.clear();
    }
}
