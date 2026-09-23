//! 黑白终端风格：字体、线框、反色标签、刻度。

use std::sync::Arc;

use eframe::egui;

pub fn install_cjk_font(ctx: &egui::Context) {
    apply_terminal_style(ctx);
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
        fonts
            .font_data
            .insert("cjk".to_owned(), Arc::new(egui::FontData::from_owned(bytes)));
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

fn apply_terminal_style(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::light();
    visuals.window_fill = egui::Color32::WHITE;
    visuals.panel_fill = egui::Color32::WHITE;
    visuals.extreme_bg_color = egui::Color32::WHITE;
    visuals.faint_bg_color = egui::Color32::from_gray(245);
    visuals.code_bg_color = egui::Color32::WHITE;
    visuals.window_stroke = egui::Stroke::new(2.0, egui::Color32::BLACK);
    visuals.window_corner_radius = egui::CornerRadius::ZERO;
    visuals.menu_corner_radius = egui::CornerRadius::ZERO;
    visuals.window_shadow = egui::Shadow::NONE;
    visuals.popup_shadow = egui::Shadow::NONE;
    visuals.selection.bg_fill = egui::Color32::BLACK;
    visuals.selection.stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);
    let stroke = egui::Stroke::new(1.5, egui::Color32::BLACK);
    for widget in [
        &mut visuals.widgets.noninteractive,
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
        &mut visuals.widgets.open,
    ] {
        widget.corner_radius = egui::CornerRadius::ZERO;
        widget.bg_stroke = stroke;
        widget.fg_stroke = egui::Stroke::new(1.0, egui::Color32::BLACK);
        widget.bg_fill = egui::Color32::WHITE;
    }
    visuals.widgets.hovered.bg_fill = egui::Color32::from_gray(230);
    visuals.widgets.active.bg_fill = egui::Color32::BLACK;
    visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);
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
}

pub(super) fn ruled_frame() -> egui::Frame {
    egui::Frame::new()
        .fill(egui::Color32::WHITE)
        .stroke(egui::Stroke::new(2.0, egui::Color32::BLACK))
        .inner_margin(10)
        .corner_radius(egui::CornerRadius::ZERO)
}

pub(super) fn ink_bar(ui: &mut egui::Ui, left: &str, right: &str) {
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 22.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, 0.0, egui::Color32::BLACK);
    ui.painter().text(
        rect.left_center() + egui::vec2(8.0, 0.0),
        egui::Align2::LEFT_CENTER,
        left,
        egui::FontId::monospace(13.0),
        egui::Color32::WHITE,
    );
    ui.painter().text(
        rect.right_center() + egui::vec2(-8.0, 0.0),
        egui::Align2::RIGHT_CENTER,
        right,
        egui::FontId::monospace(12.0),
        egui::Color32::WHITE,
    );
}

pub(super) fn ink_badge(ui: &mut egui::Ui, text: &str) {
    let galley = ui.painter().layout_no_wrap(
        text.to_owned(),
        egui::FontId::monospace(13.0),
        egui::Color32::WHITE,
    );
    let pad = egui::vec2(6.0, 2.0);
    let (rect, _) = ui.allocate_exact_size(galley.size() + pad * 2.0, egui::Sense::hover());
    ui.painter().rect_filled(rect, 0.0, egui::Color32::BLACK);
    ui.painter()
        .galley(rect.min + pad, galley, egui::Color32::WHITE);
}

pub(super) fn scale_tick(ui: &mut egui::Ui, t: f32) {
    let width = (ui.available_width() - 120.0).max(48.0);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 14.0), egui::Sense::hover());
    let y = rect.center().y;
    ui.painter()
        .hline(rect.x_range(), y, egui::Stroke::new(1.0, egui::Color32::BLACK));
    let x = rect.left() + rect.width() * t.clamp(0.04, 0.96);
    let mark = egui::Rect::from_center_size(egui::pos2(x, y), egui::vec2(7.0, 7.0));
    ui.painter().rect_filled(mark, 0.0, egui::Color32::BLACK);
}

pub(super) fn tick_at(key: &str) -> f32 {
    let hash = key
        .bytes()
        .fold(0u32, |acc, byte| acc.wrapping_mul(33).wrapping_add(byte as u32));
    (hash % 100) as f32 / 100.0
}

pub(super) fn hairline(ui: &mut egui::Ui) {
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 8.0), egui::Sense::hover());
    ui.painter().hline(
        rect.x_range(),
        rect.center().y,
        egui::Stroke::new(1.0, egui::Color32::BLACK),
    );
}

pub(super) fn dotted_band(ui: &mut egui::Ui) {
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 22.0), egui::Sense::hover());
    ui.painter().rect_stroke(
        rect,
        0.0,
        egui::Stroke::new(2.0, egui::Color32::BLACK),
        egui::StrokeKind::Inside,
    );
    let mut x = rect.left() + 8.0;
    let y = rect.center().y;
    while x < rect.right() - 6.0 {
        ui.painter()
            .circle_filled(egui::pos2(x, y), 1.1, egui::Color32::BLACK);
        x += 6.0;
    }
}

pub(super) fn provider_icon(ui: &mut egui::Ui, label: &str) {
    let Some(name) = provider_icon_name(label) else {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(16.0, 16.0), egui::Sense::hover());
        ui.painter().rect_stroke(
            rect,
            0.0,
            egui::Stroke::new(1.0, egui::Color32::BLACK),
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
    let (rect, _) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());
    ui.painter().rect_stroke(
        rect,
        0.0,
        egui::Stroke::new(2.0, egui::Color32::BLACK),
        egui::StrokeKind::Inside,
    );
    const CELLS: &[&str] = &[
        "#.#.#.#",
        "#.....#",
        "#.###.#",
        "#.#.#.#",
        "#.###.#",
        "#.....#",
        "#.#.#.#",
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
                    egui::Color32::BLACK,
                );
            }
        }
    }
}
