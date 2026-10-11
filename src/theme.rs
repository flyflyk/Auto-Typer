use eframe::egui::{
    self, vec2, Color32, CornerRadius, FontData, FontDefinitions, FontFamily, Frame, Margin,
    Response, RichText, Stroke, StrokeKind, Ui, Visuals,
};
use std::sync::Arc;

pub const BG: Color32 = Color32::from_rgb(0x0E, 0x11, 0x17);
pub const PANEL: Color32 = Color32::from_rgb(0x16, 0x1B, 0x22);
pub const CARD: Color32 = Color32::from_rgb(0x21, 0x28, 0x33);
pub const CARD_HOVER: Color32 = Color32::from_rgb(0x2B, 0x34, 0x41);
pub const CARD_PRESS: Color32 = Color32::from_rgb(0x1A, 0x20, 0x29);
pub const BORDER: Color32 = Color32::from_rgb(0x30, 0x3A, 0x48);
pub const TEXT: Color32 = Color32::from_rgb(0xE6, 0xED, 0xF3);
pub const TEXT_DIM: Color32 = Color32::from_rgb(0x9A, 0xA4, 0xB2);
pub const ACCENT: Color32 = Color32::from_rgb(0x25, 0x63, 0xEB);
pub const ACCENT_HOVER: Color32 = Color32::from_rgb(0x1D, 0x4E, 0xD8);
pub const ACCENT_PRESS: Color32 = Color32::from_rgb(0x1E, 0x40, 0xAF);
pub const SUCCESS: Color32 = Color32::from_rgb(0x4A, 0xDE, 0x80);
pub const WARNING: Color32 = Color32::from_rgb(0xFA, 0xCC, 0x15);
pub const ERROR: Color32 = Color32::from_rgb(0xF8, 0x71, 0x71);

pub const R_SM: u8 = 6;
pub const R_LG: u8 = 12;
pub const HIT: f32 = 40.0;

pub fn apply(ctx: &egui::Context) {
    let mut v = Visuals::dark();
    v.panel_fill = BG;
    v.window_fill = CARD;
    v.extreme_bg_color = BG;
    v.faint_bg_color = PANEL;
    v.selection.bg_fill = ACCENT;
    v.selection.stroke = Stroke::new(1.0_f32, TEXT);
    v.window_corner_radius = CornerRadius::same(R_LG);
    v.menu_corner_radius = CornerRadius::same(R_SM);

    let w = &mut v.widgets;
    for s in [
        &mut w.noninteractive,
        &mut w.inactive,
        &mut w.hovered,
        &mut w.active,
        &mut w.open,
    ] {
        s.corner_radius = CornerRadius::same(R_SM);
        s.expansion = 0.0;
    }
    w.noninteractive.bg_fill = PANEL;
    w.noninteractive.weak_bg_fill = PANEL;
    w.noninteractive.bg_stroke = Stroke::new(1.0_f32, BORDER);
    w.noninteractive.fg_stroke = Stroke::new(1.0_f32, TEXT);
    w.inactive.bg_fill = CARD;
    w.inactive.weak_bg_fill = CARD;
    w.inactive.bg_stroke = Stroke::new(1.0_f32, BORDER);
    w.inactive.fg_stroke = Stroke::new(1.0_f32, TEXT);
    w.hovered.bg_fill = CARD_HOVER;
    w.hovered.weak_bg_fill = CARD_HOVER;
    w.hovered.bg_stroke = Stroke::new(1.0_f32, BORDER);
    w.hovered.fg_stroke = Stroke::new(1.0_f32, TEXT);
    w.active.bg_fill = CARD_PRESS;
    w.active.weak_bg_fill = CARD_PRESS;
    w.active.bg_stroke = Stroke::new(1.0_f32, BORDER);
    w.active.fg_stroke = Stroke::new(1.0_f32, TEXT);
    w.open = w.active;

    ctx.style_mut(|s| {
        s.visuals = v;
        s.spacing.item_spacing = vec2(8.0, 8.0);
        s.spacing.button_padding = vec2(16.0, 8.0);
        s.spacing.interact_size = vec2(HIT, HIT);
        s.spacing.slider_width = 240.0;
        s.animation_time = 0.2;
    });
}

pub fn load_cjk_font(ctx: &egui::Context) {
    let paths = [
        "C:\\Windows\\Fonts\\msjh.ttc",
        "C:\\Windows\\Fonts\\msyh.ttc",
    ];
    let Some(bytes) = paths.iter().find_map(|p| std::fs::read(p).ok()) else {
        return;
    };
    let mut fonts = FontDefinitions::default();
    fonts
        .font_data
        .insert("cjk".into(), Arc::new(FontData::from_owned(bytes)));
    for fam in [FontFamily::Proportional, FontFamily::Monospace] {
        fonts.families.entry(fam).or_default().push("cjk".into());
    }
    ctx.set_fonts(fonts);
}

pub fn section<R>(ui: &mut Ui, title: &str, hot: bool, add: impl FnOnce(&mut Ui) -> R) -> R {
    let stroke = if hot {
        Stroke::new(2.0_f32, ACCENT)
    } else {
        Stroke::new(1.0_f32, BORDER)
    };
    Frame::new()
        .fill(PANEL)
        .stroke(stroke)
        .corner_radius(CornerRadius::same(R_LG))
        .inner_margin(Margin::same(16))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new(title).size(13.0).strong().color(TEXT_DIM));
            ui.add_space(8.0);
            add(ui)
        })
        .inner
}

pub fn card<R>(ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> R {
    Frame::new()
        .fill(CARD)
        .corner_radius(CornerRadius::same(R_SM))
        .inner_margin(Margin::same(16))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            add(ui)
        })
        .inner
}

pub fn focus_ring(ui: &Ui, r: &Response) {
    if r.has_focus() {
        ui.painter().rect_stroke(
            r.rect.expand(2.0),
            CornerRadius::same(R_SM + 2),
            Stroke::new(2.0_f32, ACCENT),
            StrokeKind::Outside,
        );
    }
}

pub fn button(ui: &mut Ui, text: &str, enabled: bool, primary: bool, width: f32) -> Response {
    ui.scope(|ui| {
        if primary {
            let w = &mut ui.visuals_mut().widgets;
            w.inactive.weak_bg_fill = ACCENT;
            w.hovered.weak_bg_fill = ACCENT_HOVER;
            w.active.weak_bg_fill = ACCENT_PRESS;
            for s in [&mut w.inactive, &mut w.hovered, &mut w.active] {
                s.bg_stroke = Stroke::NONE;
                s.fg_stroke = Stroke::new(1.0_f32, Color32::WHITE);
            }
        }
        let label = RichText::new(text).size(14.0).strong();
        let r = ui.add_enabled(
            enabled,
            egui::Button::new(label).min_size(vec2(width, HIT)),
        );
        focus_ring(ui, &r);
        r
    })
    .inner
}
