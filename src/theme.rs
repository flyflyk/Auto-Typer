// Visual system, from the design-taste-frontend skill's app-screen rules, style
// "true-black-flat" (seed 2089919167), shell "no sidebar".
//
// - Elevation by lightness alone: #000 ground, #111 panel, #2a2a2a nested. No borders
//   or shadows on panels; 1px rules only between the pinned header and the content
//   and between settings sections.
// - Colour only on data. The one hue is the product's own blue (its icon): the typing
//   caret, the progress, the countdown figure, the chosen part of a slider. It never
//   touches buttons, tabs or the switch. Focus rings use it too.
// - The primary action is made primary by inversion: a white 36px pill, black label.
// - One typeface: Inter 400/500/600, embedded, with tabular figures made the default
//   (egui cannot switch OpenType features on). Consolas only for the file preview,
//   which is the file's own text.
// - Radii: 16 panels, 10 nested boxes, full pills for controls.

use eframe::egui::{
    self, epaint, pos2, vec2, Align2, Color32, CornerRadius, FontData, FontDefinitions,
    FontFamily, FontId, Frame, Margin, Rect, Response, RichText, Sense, Shadow, Stroke,
    StrokeKind, TextStyle, Ui, Visuals, WidgetInfo, WidgetType,
};
use std::sync::Arc;

pub const GROUND: Color32 = Color32::from_rgb(0x00, 0x00, 0x00);
pub const SURFACE: Color32 = Color32::from_rgb(0x11, 0x11, 0x11);
pub const RAISED: Color32 = Color32::from_rgb(0x2A, 0x2A, 0x2A);
pub const RULE: Color32 = Color32::from_rgb(0x1A, 0x1A, 0x1A);
pub const CONTROL: Color32 = Color32::from_rgb(0x44, 0x44, 0x44);

// Grey-filled secondary control, one step either way for hover and press.
pub const FILL: Color32 = Color32::from_rgb(0x22, 0x22, 0x22);
pub const FILL_HOVER: Color32 = Color32::from_rgb(0x2E, 0x2E, 0x2E);
pub const FILL_PRESS: Color32 = Color32::from_rgb(0x1A, 0x1A, 0x1A);

pub const TEXT: Color32 = Color32::from_rgb(0xFF, 0xFF, 0xFF);
pub const MUTED: Color32 = Color32::from_rgb(0xB4, 0xB4, 0xB4);
pub const FAINT: Color32 = Color32::from_rgb(0x94, 0x94, 0x94);

pub const ACCENT: Color32 = Color32::from_rgb(0x3B, 0x82, 0xF6);

pub const PRIMARY: Color32 = Color32::from_rgb(0xFF, 0xFF, 0xFF);
pub const PRIMARY_HOVER: Color32 = Color32::from_rgb(0xE6, 0xE6, 0xE6);
pub const PRIMARY_PRESS: Color32 = Color32::from_rgb(0xCC, 0xCC, 0xCC);
pub const ON_PRIMARY: Color32 = Color32::from_rgb(0x00, 0x00, 0x00);

pub const POSITIVE: Color32 = Color32::from_rgb(0x3F, 0xC0, 0x7A);
pub const NEGATIVE: Color32 = Color32::from_rgb(0xE5, 0x56, 0x6F);
pub const WARNING: Color32 = Color32::from_rgb(0xE5, 0xA9, 0x3B);

pub const R_PANEL: u8 = 16;
pub const R_NESTED: u8 = 10;
pub const R_PILL: u8 = 18;

pub const CONTROL_H: f32 = 36.0;
pub const GAP: f32 = 14.0;
pub const PAD: i8 = 22;
pub const PANEL_PAD: i8 = 16;

pub const FS_TITLE: f32 = 20.0;
pub const FS_SECTION: f32 = 16.0;
pub const FS_FIGURE: f32 = 24.0;
pub const FS_BODY: f32 = 14.0;
pub const FS_SMALL: f32 = 12.0;
pub const FS_MONO: f32 = 13.0;

const MEDIUM: &str = "medium";
const SEMIBOLD: &str = "semibold";

pub fn medium(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(MEDIUM.into()))
}

pub fn semibold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(SEMIBOLD.into()))
}

pub fn apply(ctx: &egui::Context) {
    let mut v = Visuals::dark();
    v.panel_fill = GROUND;
    v.window_fill = SURFACE;
    v.window_stroke = Stroke::NONE;
    v.extreme_bg_color = SURFACE;
    v.faint_bg_color = SURFACE;
    v.code_bg_color = RAISED;
    v.hyperlink_color = ACCENT;
    v.selection.bg_fill = ACCENT;
    v.selection.stroke = Stroke::new(2.0_f32, ACCENT);
    v.window_corner_radius = CornerRadius::same(R_PANEL);
    v.menu_corner_radius = CornerRadius::same(R_NESTED);
    v.window_shadow = Shadow::NONE;
    // Only what floats (tooltips) casts a shadow.
    v.popup_shadow = Shadow {
        offset: [0, 4],
        blur: 12,
        spread: 0,
        color: Color32::from_black_alpha(140),
    };

    let w = &mut v.widgets;
    for s in [
        &mut w.noninteractive,
        &mut w.inactive,
        &mut w.hovered,
        &mut w.active,
        &mut w.open,
    ] {
        s.corner_radius = CornerRadius::same(R_NESTED);
        s.expansion = 0.0;
        s.bg_stroke = Stroke::NONE;
    }
    w.noninteractive.bg_fill = SURFACE;
    w.noninteractive.weak_bg_fill = SURFACE;
    w.noninteractive.fg_stroke = Stroke::new(1.0_f32, MUTED);
    w.inactive.bg_fill = FILL;
    w.inactive.weak_bg_fill = FILL;
    w.inactive.fg_stroke = Stroke::new(1.0_f32, TEXT);
    w.hovered.bg_fill = FILL_HOVER;
    w.hovered.weak_bg_fill = FILL_HOVER;
    w.hovered.fg_stroke = Stroke::new(1.0_f32, TEXT);
    w.active.bg_fill = FILL_PRESS;
    w.active.weak_bg_fill = FILL_PRESS;
    w.active.fg_stroke = Stroke::new(1.0_f32, TEXT);
    w.open = w.active;

    ctx.style_mut(|s| {
        s.visuals = v;
        s.text_styles = [
            (TextStyle::Heading, semibold(FS_TITLE)),
            (TextStyle::Body, FontId::proportional(FS_BODY)),
            (TextStyle::Button, medium(FS_BODY)),
            (TextStyle::Small, FontId::proportional(FS_SMALL)),
            (TextStyle::Monospace, FontId::monospace(FS_MONO)),
        ]
        .into();
        s.spacing.item_spacing = vec2(8.0, 6.0);
        s.spacing.button_padding = vec2(16.0, 8.0);
        s.spacing.interact_size = vec2(CONTROL_H, CONTROL_H);
        s.animation_time = 0.15;
    });
}

/// Inter (embedded) for every word and figure; Consolas for the file preview; Microsoft
/// JhengHei as the CJK fallback for both.
pub fn load_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    let mut put = |name: &str, bytes: Vec<u8>| {
        fonts
            .font_data
            .insert(name.into(), Arc::new(FontData::from_owned(bytes)));
    };
    put("inter", include_bytes!("../assets/fonts/Inter-Regular.otf").to_vec());
    put("inter-medium", include_bytes!("../assets/fonts/Inter-Medium.otf").to_vec());
    put("inter-semibold", include_bytes!("../assets/fonts/Inter-SemiBold.otf").to_vec());
    let read = |paths: &[&str]| paths.iter().find_map(|p| std::fs::read(p).ok());
    if let Some(b) = read(&["C:\\Windows\\Fonts\\consola.ttf"]) {
        put("mono", b);
    }
    if let Some(b) = read(&["C:\\Windows\\Fonts\\msjh.ttc", "C:\\Windows\\Fonts\\msyh.ttc"]) {
        put("cjk", b);
    }
    let has_mono = fonts.font_data.contains_key("mono");
    let has_cjk = fonts.font_data.contains_key("cjk");

    let defaults = fonts
        .families
        .get(&FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();
    let family = |first: &str| {
        let mut v = vec![first.to_string()];
        if has_cjk {
            v.push("cjk".into());
        }
        v.extend(defaults.iter().cloned());
        v
    };
    fonts.families.insert(FontFamily::Proportional, family("inter"));
    fonts.families.insert(FontFamily::Name(MEDIUM.into()), family("inter-medium"));
    fonts.families.insert(FontFamily::Name(SEMIBOLD.into()), family("inter-semibold"));
    let monofam = fonts.families.entry(FontFamily::Monospace).or_default();
    if has_mono {
        monofam.insert(0, "mono".into());
    }
    if has_cjk {
        monofam.push("cjk".into());
    }
    ctx.set_fonts(fonts);
}

pub fn panel<R>(ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> R {
    Frame::new()
        .fill(SURFACE)
        .corner_radius(CornerRadius::same(R_PANEL))
        .inner_margin(Margin::same(PANEL_PAD))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            add(ui)
        })
        .inner
}

/// A full-width 1px rule.
pub fn rule(ui: &mut Ui) {
    let w = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(vec2(w, 1.0), Sense::hover());
    ui.painter().rect_filled(rect, CornerRadius::ZERO, RULE);
}

/// A settings section's label: a 16/500 title and a 14px grey description.
pub fn section_label(ui: &mut Ui, title: &str, desc: &str) {
    ui.label(RichText::new(title).font(medium(FS_SECTION)).color(TEXT));
    ui.add(egui::Label::new(RichText::new(desc).size(FS_BODY).color(MUTED)).wrap());
}

pub fn note(ui: &mut Ui, text: &str) {
    ui.add(egui::Label::new(RichText::new(text).size(FS_SMALL).color(FAINT)).wrap());
}

fn ring(ui: &Ui, rect: Rect, radius: u8) {
    ui.painter().rect_stroke(
        rect.expand(2.0),
        CornerRadius::same(radius.saturating_add(2)),
        Stroke::new(2.0_f32, ACCENT),
        StrokeKind::Outside,
    );
}

#[derive(Clone, Copy, PartialEq)]
pub enum Kind {
    Primary,
    Secondary,
}

/// A 36px pill with its label centred; `width` 0 sizes it to the label.
pub fn button(ui: &mut Ui, text: &str, enabled: bool, kind: Kind, width: f32) -> Response {
    let font = medium(FS_BODY);
    let galley = ui.painter().layout_no_wrap(text.to_string(), font.clone(), TEXT);
    let w = width.max(galley.size().x + 36.0);
    let sense = if enabled { Sense::click() } else { Sense::hover() };
    let (rect, r) = ui.allocate_exact_size(vec2(w, CONTROL_H), sense);
    r.widget_info(|| WidgetInfo::labeled(WidgetType::Button, enabled, text));
    let down = r.is_pointer_button_down_on();
    let (fill, ink) = match (kind, enabled) {
        (_, false) => (FILL, FAINT),
        (Kind::Primary, _) if down => (PRIMARY_PRESS, ON_PRIMARY),
        (Kind::Primary, _) if r.hovered() => (PRIMARY_HOVER, ON_PRIMARY),
        (Kind::Primary, _) => (PRIMARY, ON_PRIMARY),
        (Kind::Secondary, _) if down => (FILL_PRESS, TEXT),
        (Kind::Secondary, _) if r.hovered() => (FILL_HOVER, TEXT),
        (Kind::Secondary, _) => (FILL, TEXT),
    };
    let p = ui.painter();
    p.rect_filled(rect, CornerRadius::same(R_PILL), fill);
    p.text(rect.center(), Align2::CENTER_CENTER, text, font, ink);
    if enabled && r.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    if r.has_focus() {
        ring(ui, rect, R_PILL);
    }
    r
}

/// A setting's label with its value at the right: the number in ink, the unit grey.
pub fn value_row(ui: &mut Ui, label: &str, value: u32, unit: &str, enabled: bool) {
    let ink = if enabled { TEXT } else { FAINT };
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).font(medium(FS_BODY)).color(ink));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.spacing_mut().item_spacing.x = 3.0;
            ui.label(RichText::new(unit).size(FS_SMALL).color(FAINT));
            ui.label(RichText::new(value.to_string()).size(FS_BODY).color(ink));
        });
    });
}

/// A full-width slider: the chosen part in the data hue on a nested-grey rail, a white
/// knob. Click or drag to set; arrow keys step by 1, Page Up/Down by 10.
pub fn slider(ui: &mut Ui, value: &mut u32, max: u32, enabled: bool) -> Response {
    let w = ui.available_width();
    let sense = if enabled { Sense::click_and_drag() } else { Sense::hover() };
    let (rect, mut r) = ui.allocate_exact_size(vec2(w, 24.0), sense);
    let knob_r = 8.0;
    let (left, right) = (rect.left() + knob_r, rect.right() - knob_r);
    if enabled {
        if let Some(pos) = r.interact_pointer_pos() {
            let t = ((pos.x - left) / (right - left)).clamp(0.0, 1.0);
            let v = (t * max as f32).round() as u32;
            if v != *value {
                *value = v;
                r.mark_changed();
            }
        }
        if r.has_focus() {
            let step = ui.input(|i| {
                let mut d: i64 = 0;
                if i.key_pressed(egui::Key::ArrowRight) || i.key_pressed(egui::Key::ArrowUp) {
                    d += 1;
                }
                if i.key_pressed(egui::Key::ArrowLeft) || i.key_pressed(egui::Key::ArrowDown) {
                    d -= 1;
                }
                if i.key_pressed(egui::Key::PageUp) {
                    d += 10;
                }
                if i.key_pressed(egui::Key::PageDown) {
                    d -= 10;
                }
                d
            });
            if step != 0 {
                *value = (*value as i64 + step).clamp(0, max as i64) as u32;
                r.mark_changed();
            }
        }
        if r.hovered() || r.dragged() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
    }
    let val = *value;
    r.widget_info(|| WidgetInfo::slider(enabled, val as f64, ""));
    let t = if max == 0 { 0.0 } else { val as f32 / max as f32 };
    let x = egui::lerp(left..=right, t);
    let cy = rect.center().y;
    let p = ui.painter();
    let rail = Rect::from_min_max(pos2(left, cy - 2.0), pos2(right, cy + 2.0));
    p.rect_filled(rail, CornerRadius::same(2), RAISED);
    let done = Rect::from_min_max(rail.min, pos2(x, rail.max.y));
    p.rect_filled(done, CornerRadius::same(2), if enabled { ACCENT } else { CONTROL });
    let knob = match (enabled, r.dragged() || r.is_pointer_button_down_on(), r.hovered()) {
        (false, _, _) => FAINT,
        (true, true, _) => PRIMARY_PRESS,
        (true, false, true) => PRIMARY_HOVER,
        _ => PRIMARY,
    };
    p.circle_filled(pos2(x, cy), knob_r, knob);
    if r.has_focus() {
        p.circle_stroke(pos2(x, cy), knob_r + 3.0, Stroke::new(2.0_f32, ACCENT));
    }
    r
}

/// A switch row: a 14/500 title and a grey description on the left, a 28x16 switch on
/// the right centred on the title line. On is white (inverted), never the data hue.
pub fn switch_row(ui: &mut Ui, on: &mut bool, title: &str, desc: &str) -> Response {
    let w = ui.available_width();
    let text_w = w - 28.0 - 16.0;
    let desc_galley = ui
        .painter()
        .layout(desc.to_string(), FontId::proportional(FS_BODY), MUTED, text_w);
    let h = 24.0 + desc_galley.size().y;
    let (rect, mut r) = ui.allocate_exact_size(vec2(w, h.max(CONTROL_H)), Sense::click());
    if r.clicked() {
        *on = !*on;
        r.mark_changed();
    }
    let enabled = ui.is_enabled();
    r.widget_info(|| WidgetInfo::selected(WidgetType::Checkbox, enabled, *on, title));
    let p = ui.painter();
    let title_y = rect.top() + 10.0;
    p.text(
        pos2(rect.left(), title_y),
        Align2::LEFT_CENTER,
        title,
        medium(FS_BODY),
        if enabled { TEXT } else { FAINT },
    );
    p.galley(pos2(rect.left(), rect.top() + 24.0), desc_galley, MUTED);
    let track = Rect::from_center_size(pos2(rect.right() - 14.0, title_y), vec2(28.0, 16.0));
    let t = ui.ctx().animate_bool_responsive(r.id, *on);
    let (fill, knob) = match (enabled, *on) {
        (false, _) => (RAISED, FAINT),
        (true, true) => (if r.hovered() { PRIMARY_HOVER } else { PRIMARY }, ON_PRIMARY),
        (true, false) => (if r.hovered() { FILL_HOVER } else { CONTROL }, MUTED),
    };
    p.rect_filled(track, CornerRadius::same(8), fill);
    let x = egui::lerp(track.left() + 8.0..=track.right() - 8.0, t);
    p.circle_filled(pos2(x, track.center().y), 6.0, knob);
    if r.hovered() && enabled {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    if r.has_focus() {
        ring(ui, track, 8);
    }
    r
}

/// A segmented control as wide as its options: a 1px control outline, the chosen
/// option a nested-grey pill with white text, the others grey text.
pub fn segmented<T: PartialEq + Copy>(ui: &mut Ui, value: &mut T, options: &[(T, &str)]) -> Response {
    let font = medium(FS_BODY);
    let seg_w = options
        .iter()
        .map(|(_, l)| ui.painter().layout_no_wrap(l.to_string(), font.clone(), TEXT).size().x + 36.0)
        .fold(0.0, f32::max);
    let n = options.len() as f32;
    let (rect, mut outer) =
        ui.allocate_exact_size(vec2(seg_w * n + 8.0, CONTROL_H + 4.0), Sense::hover());
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(R_PILL + 2),
        Stroke::new(1.0_f32, CONTROL),
        StrokeKind::Inside,
    );
    let enabled = ui.is_enabled();
    for (i, (opt, label)) in options.iter().enumerate() {
        let seg = Rect::from_min_size(
            pos2(rect.left() + 4.0 + seg_w * i as f32, rect.top() + 4.0),
            vec2(seg_w, rect.height() - 8.0),
        );
        let r = ui.interact(seg, outer.id.with(i), Sense::click());
        let selected = *value == *opt;
        r.widget_info(|| WidgetInfo::selected(WidgetType::RadioButton, enabled, selected, *label));
        if r.clicked() && !selected {
            *value = *opt;
            outer.mark_changed();
        }
        let fill = if selected {
            RAISED
        } else if r.is_pointer_button_down_on() {
            FILL_PRESS
        } else if r.hovered() {
            FILL
        } else {
            Color32::TRANSPARENT
        };
        let p = ui.painter();
        p.rect_filled(seg, CornerRadius::same(R_PILL - 2), fill);
        let ink = match (enabled, selected) {
            (false, _) => FAINT,
            (true, true) => TEXT,
            (true, false) => MUTED,
        };
        p.text(seg.center(), Align2::CENTER_CENTER, *label, font.clone(), ink);
        if r.hovered() && enabled && !selected {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        if r.has_focus() {
            ring(ui, seg, R_PILL - 2);
        }
    }
    outer
}

/// A 4px progress bar in the data hue on a nested-grey track.
pub fn progress(ui: &mut Ui, f: f32) {
    let w = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(vec2(w, 4.0), Sense::hover());
    let p = ui.painter();
    p.rect_filled(rect, CornerRadius::same(2), RAISED);
    let mut done = rect;
    done.set_width(rect.width() * f.clamp(0.0, 1.0));
    p.rect_filled(done, CornerRadius::same(2), ACCENT);
}

/// A 1px dashed outline: the empty slot a file is dropped into.
pub fn dashed_rect(ui: &Ui, rect: Rect, color: Color32) {
    let s = Stroke::new(1.0_f32, color);
    let pts = [
        rect.left_top(),
        rect.right_top(),
        rect.right_bottom(),
        rect.left_bottom(),
        rect.left_top(),
    ];
    let shapes = pts
        .windows(2)
        .flat_map(|w| epaint::Shape::dashed_line(&[w[0], w[1]], s, 6.0, 4.0))
        .collect::<Vec<_>>();
    ui.painter().extend(shapes);
}

/// The product mark, drawn from the icon: the I-beam caret in the product's blue with
/// its speed lines.
pub fn mark(ui: &mut Ui, size: f32) {
    let (rect, _) = ui.allocate_exact_size(vec2(size, size), Sense::hover());
    let p = ui.painter();
    let u = size / 24.0;
    let cx = rect.left() + 16.0 * u;
    let top = rect.top() + 3.0 * u;
    let bot = rect.bottom() - 3.0 * u;
    let beam = Stroke::new(2.4 * u, ACCENT);
    p.line_segment([pos2(cx, top), pos2(cx, bot)], beam);
    p.line_segment([pos2(cx - 3.5 * u, top), pos2(cx + 3.5 * u, top)], beam);
    p.line_segment([pos2(cx - 3.5 * u, bot), pos2(cx + 3.5 * u, bot)], beam);
    let speed = Stroke::new(2.0 * u, MUTED);
    for (y, len) in [(8.0, 6.0), (12.0, 9.0), (16.0, 5.0)] {
        let y = rect.top() + y * u;
        let x1 = cx - 5.0 * u;
        p.line_segment([pos2(x1 - len * u, y), pos2(x1, y)], speed);
    }
}

pub fn thousands(n: usize) -> String {
    let s = n.to_string();
    let mut out = String::with_capacity(s.len() + s.len() / 3);
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// A duration in words for the estimate: "under 1 s", "4 s", "2 min 10 s", "1 h 5 min".
pub fn duration(ms: u64) -> String {
    let s = ms.div_ceil(1000);
    match s {
        0 => "under 1 s".into(),
        1..=59 => format!("{s} s"),
        60..=3599 if s.is_multiple_of(60) => format!("{} min", s / 60),
        60..=3599 => format!("{} min {} s", s / 60, s % 60),
        _ => format!("{} h {} min", s / 3600, (s % 3600) / 60),
    }
}
