#![windows_subsystem = "windows"]

mod theme;
mod typer;
mod update;

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use eframe::egui::{
    self, text::LayoutJob, text::TextWrapping, Align, Color32, Frame, Label, Layout, Margin,
    RichText, Sense, TextFormat, Ui,
};
use theme::{
    Kind, ACCENT, CONTROL, FAINT, FS_BODY, FS_FIGURE, FS_MONO, FS_SMALL, FS_TITLE, GAP, GROUND,
    MUTED, NEGATIVE, PAD, POSITIVE, TEXT, WARNING,
};
use typer::{Job, Mode, Phase};
use update::Update;

const COUNTDOWN_S: u32 = 5;
const PREVIEW_ROWS: usize = 10;
// Characters that need Shift on a US layout; each costs two Shift gaps in Keystrokes mode.
const SHIFTED: &str = "~!@#$%^&*()_+{}|:\"<>?";

enum Src {
    Empty,
    Ready { path: PathBuf, text: String },
    Failed { msg: String },
}

/// The text exactly as it will be typed (after CRLF folding and optional indent
/// stripping), with the index of each line's first character for the preview.
#[derive(Default)]
struct Prepared {
    chars: Vec<char>,
    starts: Vec<usize>,
    shifted: usize,
    strip: bool,
}

impl Prepared {
    fn new(text: &str, strip: bool) -> Self {
        let chars = typer::prepare(text, strip);
        let mut starts = vec![0];
        starts.extend(chars.iter().enumerate().filter(|(_, c)| **c == '\n').map(|(i, _)| i + 1));
        let shifted = chars
            .iter()
            .filter(|c| c.is_ascii_uppercase() || SHIFTED.contains(**c))
            .count();
        Self { chars, starts, shifted, strip }
    }

    fn lines(&self) -> usize {
        let trailing = matches!(self.chars.last(), Some('\n'));
        self.starts.len() - usize::from(trailing)
    }

    /// Typing time from the settings alone (the keystrokes themselves add a little).
    fn estimate_ms(&self, delay_ms: u32, shift_gap_ms: u32, mode: Mode) -> u64 {
        let base = self.chars.len() as u64 * delay_ms as u64;
        let gaps = if mode == Mode::Keys { self.shifted as u64 * 2 * shift_gap_ms as u64 } else { 0 };
        base + gaps
    }
}

struct App {
    src: Src,
    prepared: Prepared,
    delay_ms: u32,
    shift_gap_ms: u32,
    mode: Mode,
    strip: bool,
    phase: Arc<Mutex<Phase>>,
    stop: Arc<AtomicBool>,
    update: Arc<Mutex<Update>>,
    old_copy: Option<PathBuf>,
    old_copy_error: Option<String>,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::load_fonts(&cc.egui_ctx);
        theme::apply(&cc.egui_ctx);
        let update = Arc::new(Mutex::new(Update::Idle));
        update::check(update.clone(), cc.egui_ctx.clone());
        Self {
            src: Src::Empty,
            prepared: Prepared::default(),
            delay_ms: 15,
            shift_gap_ms: 0,
            mode: Mode::Keys,
            strip: false,
            phase: Arc::new(Mutex::new(Phase::Idle)),
            stop: Arc::new(AtomicBool::new(false)),
            update,
            old_copy: update::migrated_from(),
            old_copy_error: None,
        }
    }

    fn load(&mut self, path: PathBuf) {
        self.src = match std::fs::read(&path) {
            Err(e) => Src::Failed { msg: format!("Couldn't read {}: {e}", file_name(&path)) },
            Ok(b) if b.is_empty() => Src::Failed { msg: format!("{} is empty.", file_name(&path)) },
            Ok(b) => match String::from_utf8(b) {
                Ok(text) => Src::Ready { path, text },
                Err(_) => Src::Failed {
                    msg: format!("{} isn't UTF-8 text. Save it as UTF-8 and try again.", file_name(&path)),
                },
            },
        };
        self.prepared = match &self.src {
            Src::Ready { text, .. } => Prepared::new(text, self.strip),
            _ => Prepared::default(),
        };
        *self.phase.lock().unwrap() = Phase::Idle;
    }

    fn sync_prepared(&mut self) {
        if let Src::Ready { text, .. } = &self.src {
            if self.prepared.strip != self.strip {
                self.prepared = Prepared::new(text, self.strip);
            }
        }
    }

    fn choose_file(&mut self) {
        if let Some(p) = pick_file() {
            self.load(p);
        }
    }

    fn start(&mut self, ctx: &egui::Context) {
        if !matches!(self.src, Src::Ready { .. }) {
            return;
        }
        let job = Job {
            chars: self.prepared.chars.clone(),
            delay_ms: self.delay_ms,
            shift_gap_ms: self.shift_gap_ms,
            mode: self.mode,
            countdown_s: COUNTDOWN_S,
        };
        self.stop.store(false, Ordering::Relaxed);
        *self.phase.lock().unwrap() = Phase::Countdown(COUNTDOWN_S);
        typer::spawn(job, self.phase.clone(), self.stop.clone(), ctx.clone());
    }

    /// The 48px bar: the product's mark and name on the column's left edge, its version
    /// (or the update action) at the right.
    fn top_bar(&mut self, ui: &mut Ui, ctx: &egui::Context, busy: bool) {
        let state = self.update.lock().unwrap().clone();
        let mut go = None;
        ui.horizontal_centered(|ui| {
            theme::mark(ui, 20.0);
            ui.add_space(2.0);
            ui.label(RichText::new(update::APP_NAME).font(theme::semibold(FS_BODY)).color(TEXT));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| match &state {
                Update::Offer { rel, install: false } => {
                    let label = format!("Update to {}", rel.version);
                    let r = theme::button(ui, &label, !busy, Kind::Secondary, 0.0);
                    let r = if busy { r.on_hover_text("Available when typing has finished.") } else { r };
                    if r.clicked() {
                        go = Some((rel.clone(), false));
                    }
                    ui.label(RichText::new(format!("v{}", update::current())).size(FS_SMALL).color(FAINT));
                }
                Update::Working(msg) => {
                    ui.label(RichText::new(msg).size(FS_SMALL).color(MUTED));
                    ui.spinner();
                }
                _ => {
                    ui.label(RichText::new(format!("v{}", update::current())).size(FS_SMALL).color(FAINT));
                }
            });
        });
        if let Some((rel, install)) = go {
            update::run(rel, install, self.update.clone(), ctx.clone());
        }
    }

    /// The title row: the file is the h1, the line under it is the finding (what will be
    /// typed and how long it takes, or where typing is), and Start sits at its right.
    fn title_row(&mut self, ui: &mut Ui, ctx: &egui::Context, phase: &Phase) {
        let busy = phase.busy();
        let ready = matches!(self.src, Src::Ready { .. });
        let mut choose = false;
        let mut start = false;
        ui.horizontal(|ui| {
            let actions_w = if busy { 84.0 } else if ready { 196.0 } else { 140.0 };
            ui.vertical(|ui| {
                ui.set_max_width(ui.available_width() - actions_w);
                let title = match &self.src {
                    Src::Ready { path, .. } => file_name(path),
                    _ => "No file chosen".into(),
                };
                let r = ui.add(
                    Label::new(RichText::new(&title).font(theme::semibold(FS_TITLE)).color(TEXT)).truncate(),
                );
                if let Src::Ready { path, .. } = &self.src {
                    r.on_hover_text(path.display().to_string());
                }
                match &self.src {
                    Src::Ready { .. } if !busy => {
                        let est = self.prepared.estimate_ms(self.delay_ms, self.shift_gap_ms, self.mode);
                        ui.label(
                            RichText::new(format!(
                                "{} characters · {} lines · about {} to type",
                                theme::thousands(self.prepared.chars.len()),
                                theme::thousands(self.prepared.lines()),
                                theme::duration(est)
                            ))
                            .color(MUTED),
                        );
                    }
                    Src::Ready { .. } => {}
                    Src::Empty => {
                        ui.label(RichText::new("Drop a UTF-8 text file onto this window, or choose one.").color(MUTED));
                    }
                    Src::Failed { msg } => {
                        ui.add(Label::new(RichText::new(msg).color(NEGATIVE)).wrap());
                    }
                }
            });
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if busy {
                    if theme::button(ui, "Stop", true, Kind::Secondary, 0.0).clicked() {
                        self.stop.store(true, Ordering::Relaxed);
                    }
                } else if ready {
                    start = theme::button(ui, "Start", true, Kind::Primary, 84.0).clicked();
                    choose = theme::button(ui, "Change file", true, Kind::Secondary, 0.0).clicked();
                } else {
                    choose = theme::button(ui, "Choose file", true, Kind::Primary, 0.0).clicked();
                }
            });
        });
        self.status(ui, phase);
        if choose {
            self.choose_file();
        }
        if start {
            self.start(ctx);
        }
    }

    /// Where typing is. Idle: what Start will do. Nothing here is decoration: the
    /// countdown figure and the progress are the screen's data.
    fn status(&self, ui: &mut Ui, phase: &Phase) {
        match phase {
            Phase::Idle => {
                if matches!(self.src, Src::Ready { .. }) {
                    ui.label(
                        RichText::new("Start gives you 5 s to click into the target window. Esc stops typing.")
                            .size(FS_SMALL)
                            .color(FAINT),
                    );
                }
            }
            Phase::Countdown(s) => {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("{s} s")).font(theme::semibold(FS_FIGURE)).color(ACCENT));
                    ui.add_space(4.0);
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 0.0;
                        ui.label(RichText::new("until typing starts").color(TEXT));
                        ui.label(RichText::new("Click into the target window now. Esc stops.").size(FS_SMALL).color(MUTED));
                    });
                });
            }
            Phase::Typing { done, total } => {
                let f = if *total == 0 { 1.0 } else { *done as f32 / *total as f32 };
                let left = (*total - *done) as u64 * self.delay_ms as u64;
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(format!("{} of {} characters typed", theme::thousands(*done), theme::thousands(*total)))
                            .color(TEXT),
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(RichText::new(format!("about {} left · Esc stops", theme::duration(left))).color(MUTED));
                    });
                });
                theme::progress(ui, f);
            }
            Phase::Done(n) => {
                ui.label(RichText::new(format!("Typed all {} characters.", theme::thousands(*n))).color(POSITIVE));
            }
            Phase::Stopped(n) => {
                let total = self.prepared.chars.len();
                ui.label(
                    RichText::new(format!(
                        "Stopped after {} of {} characters.",
                        theme::thousands(*n),
                        theme::thousands(total)
                    ))
                    .color(WARNING),
                );
            }
        }
    }

    /// Notices that need a decision: the one-time install offer, a failed update, and
    /// the old portable copy after installing.
    fn notices(&mut self, ui: &mut Ui, ctx: &egui::Context, busy: bool) {
        let state = self.update.lock().unwrap().clone();
        let mut go = None;
        let mut dismiss = false;
        match &state {
            Update::Offer { rel, install: true } => {
                notice(
                    ui,
                    &format!("Install {}", update::APP_NAME),
                    "Adds it to the Start menu and the desktop, and keeps it up to date.",
                    MUTED,
                    |ui| {
                        let r = theme::button(ui, "Install", !busy, Kind::Secondary, 0.0);
                        let r = if busy { r.on_hover_text("Available when typing has finished.") } else { r };
                        if r.clicked() {
                            go = Some((rel.clone(), true));
                        }
                        if theme::button(ui, "Not now", true, Kind::Secondary, 0.0).clicked() {
                            dismiss = true;
                        }
                    },
                );
            }
            Update::Failed { rel, install, msg } => {
                let what = if *install { "Install" } else { "Update" };
                notice(ui, &format!("{what} failed"), msg, NEGATIVE, |ui| {
                    if theme::button(ui, "Retry", !busy, Kind::Secondary, 0.0).clicked() {
                        go = Some((rel.clone(), *install));
                    }
                });
            }
            _ => {}
        }
        if dismiss {
            update::dismiss_install(&self.update);
        }
        if let Some((rel, install)) = go {
            update::run(rel, install, self.update.clone(), ctx.clone());
        }

        if let Some(path) = self.old_copy.clone() {
            let mut close = false;
            let (sub, color) = match &self.old_copy_error {
                Some(e) => (format!("Couldn't delete the old copy: {e}"), NEGATIVE),
                None => (format!("The copy you ran before is still at {}.", path.display()), MUTED),
            };
            notice(ui, &format!("{} is installed", update::APP_NAME), &sub, color, |ui| {
                if theme::button(ui, "Delete it", true, Kind::Secondary, 0.0).clicked() {
                    match update::delete_old_copy(&path) {
                        Ok(()) => close = true,
                        Err(e) => self.old_copy_error = Some(e),
                    }
                }
                if theme::button(ui, "Keep", true, Kind::Secondary, 0.0).clicked() {
                    close = true;
                }
            });
            if close {
                self.old_copy = None;
            }
        }
    }

    /// The screen's biggest block: the file as it will be typed. Empty, it is the slot
    /// a file is dropped into.
    fn preview_block(&mut self, ui: &mut Ui, phase: &Phase, dragging: bool) {
        let busy = phase.busy();
        let mut choose = false;
        theme::panel(ui, |ui| {
            let Src::Ready { .. } = &self.src else {
                let hot = dragging && !busy;
                let (rect, r) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 220.0), Sense::click());
                let edge = if hot { ACCENT } else if r.hovered() { MUTED } else { CONTROL };
                theme::dashed_rect(ui, rect.shrink(0.5), edge);
                let c = rect.center();
                let p = ui.painter();
                let title = if hot { "Release to load the file" } else { "Drop a text file here" };
                p.text(c - egui::vec2(0.0, 11.0), egui::Align2::CENTER_CENTER, title, theme::medium(FS_BODY), TEXT);
                p.text(
                    c + egui::vec2(0.0, 11.0),
                    egui::Align2::CENTER_CENTER,
                    "or click to choose one · UTF-8 text",
                    egui::FontId::proportional(FS_SMALL),
                    FAINT,
                );
                if r.hovered() && !busy {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
                choose = r.clicked() && !busy;
                return;
            };
            let caret = match phase {
                Phase::Countdown(_) => Some(0),
                Phase::Typing { done, .. } => Some(*done),
                Phase::Done(n) | Phase::Stopped(n) => Some(*n),
                Phase::Idle => None,
            };
            let first = first_line(&self.prepared, caret);
            let last = (first + PREVIEW_ROWS).min(self.prepared.lines());
            ui.horizontal(|ui| {
                let hint = if dragging && !busy { "Release to replace the file" } else { "What will be typed" };
                ui.label(RichText::new(hint).font(theme::medium(theme::FS_SECTION)).color(TEXT));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if self.prepared.lines() > 0 {
                        ui.label(
                            RichText::new(format!(
                                "Lines {}–{} of {}",
                                theme::thousands(first + 1),
                                theme::thousands(last),
                                theme::thousands(self.prepared.lines())
                            ))
                            .size(FS_SMALL)
                            .color(FAINT),
                        );
                    }
                });
            });
            ui.add_space(6.0);
            preview(ui, &self.prepared, caret, first);
        });
        if choose {
            self.choose_file();
        }
    }

    fn settings(&mut self, ui: &mut Ui, busy: bool) {
        ui.add_enabled_ui(!busy, |ui| {
            theme::section_label(ui, "Speed", "Slow typing down if characters go missing over a remote desktop.");
            ui.add_space(10.0);
            theme::panel(ui, |ui| {
                theme::value_row(ui, "Delay per character", self.delay_ms, "ms", !busy);
                theme::slider(ui, &mut self.delay_ms, 200, !busy);
                ui.add_space(10.0);
                let keys = self.mode == Mode::Keys;
                theme::value_row(ui, "Shift gap", self.shift_gap_ms, "ms", keys && !busy);
                theme::slider(ui, &mut self.shift_gap_ms, 100, keys && !busy);
                theme::note(
                    ui,
                    if keys {
                        "The pause between Shift and the key. Raise it if capitals come out wrong over chained remote connections; 0 sends them together."
                    } else {
                        "Only used with Keystrokes: Unicode input never presses Shift."
                    },
                );
            });

            section_break(ui);
            theme::section_label(ui, "Input method", "Switch your keyboard to English before you start.");
            ui.add_space(10.0);
            theme::panel(ui, |ui| {
                theme::segmented(ui, &mut self.mode, &[(Mode::Keys, "Keystrokes"), (Mode::Unicode, "Unicode")]);
                ui.add_space(4.0);
                let what = match self.mode {
                    Mode::Keys => "Presses real keys, so it works in remote desktops, VMs and terminals. Characters a key can't produce are sent as Unicode.",
                    Mode::Unicode => "Sends each character directly and handles CJK text, but some remote desktops and VMs ignore it.",
                };
                ui.add(Label::new(RichText::new(what).color(MUTED)).wrap());
            });

            section_break(ui);
            theme::section_label(ui, "Indentation", "For editors that indent each new line by themselves.");
            ui.add_space(10.0);
            theme::panel(ui, |ui| {
                theme::switch_row(
                    ui,
                    &mut self.strip,
                    "Strip leading indentation",
                    "Removes spaces and tabs at the start of every line so the editor's indent isn't doubled. The preview shows the result.",
                );
            });
        });
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let phase = self.phase.lock().unwrap().clone();
        let busy = phase.busy();
        if *self.update.lock().unwrap() == Update::Restart {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        let (dropped, dragging) = ctx.input(|i| {
            (
                i.raw.dropped_files.first().and_then(|f| f.path.clone()),
                !i.raw.hovered_files.is_empty(),
            )
        });
        if let (Some(p), false) = (dropped, busy) {
            self.load(p);
        }

        // Pinned: the bar and the title row, so Start and the countdown never scroll
        // away. A 1px rule marks where the content scrolls under them.
        egui::TopBottomPanel::top("bar")
            .frame(Frame::new().fill(GROUND).inner_margin(Margin::symmetric(PAD, 0)))
            .exact_height(48.0)
            .show_separator_line(false)
            .show(ctx, |ui| self.top_bar(ui, ctx, busy));
        egui::TopBottomPanel::top("title")
            .frame(Frame::new().fill(GROUND).inner_margin(Margin { left: PAD, right: PAD, top: 12, bottom: GAP as i8 }))
            .show_separator_line(false)
            .show(ctx, |ui| {
                self.title_row(ui, ctx, &phase);
                ui.add_space(GAP - 6.0);
                theme::rule(ui);
            });

        egui::CentralPanel::default()
            .frame(Frame::new().fill(GROUND).inner_margin(Margin { left: PAD, right: PAD, top: 0, bottom: 0 }))
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 6.0;
                    self.notices(ui, ctx, busy);
                    self.preview_block(ui, &phase, dragging);
                    section_break(ui);
                    self.settings(ui, busy);
                    self.sync_prepared();
                    ui.add_space(32.0);
                });
            });
    }
}

/// A section split: a full-width rule with 32px above and below.
fn section_break(ui: &mut Ui) {
    ui.add_space(32.0 - 6.0);
    theme::rule(ui);
    ui.add_space(32.0 - 6.0);
}

fn notice(ui: &mut Ui, title: &str, sub: &str, sub_color: Color32, actions: impl FnOnce(&mut Ui)) {
    theme::panel(ui, |ui| {
        ui.label(RichText::new(title).font(theme::medium(FS_BODY)).color(TEXT));
        ui.add(Label::new(RichText::new(sub).color(sub_color)).wrap());
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.with_layout(Layout::right_to_left(Align::Center), actions);
        });
    });
    ui.add_space(GAP - 6.0);
}

/// The first preview row: the top for a fresh file, two lines above the caret while
/// typing, and never past the point where the last line would leave rows empty.
fn first_line(p: &Prepared, caret: Option<usize>) -> usize {
    let at = match caret {
        Some(c) => p.starts.partition_point(|&s| s <= c).saturating_sub(1).saturating_sub(2),
        None => 0,
    };
    at.min(p.lines().saturating_sub(PREVIEW_ROWS))
}

/// The file as it will be typed. While typing, the typed part is white, the rest grey,
/// and the next character sits on the data hue, so the window follows along.
fn preview(ui: &mut Ui, p: &Prepared, caret: Option<usize>, first: usize) {
    let mono = egui::FontId::monospace(FS_MONO);
    ui.spacing_mut().item_spacing.y = 2.0;
    let width = ui.available_width();
    for row in 0..PREVIEW_ROWS {
        let line = first + row;
        let mut job = LayoutJob {
            wrap: TextWrapping {
                max_width: width,
                max_rows: 1,
                break_anywhere: true,
                overflow_character: Some('…'),
            },
            ..Default::default()
        };
        if line < p.starts.len() {
            let s = p.starts[line];
            let e = p.starts.get(line + 1).map_or(p.chars.len(), |n| n - 1);
            // Keep the caret in view on long lines.
            let from = match caret {
                Some(c) if c >= s && c <= e && c - s > 48 => c - 40,
                _ => s,
            };
            let fmt = |i: usize| {
                let (color, background) = match caret {
                    None => (MUTED, Color32::TRANSPARENT),
                    Some(c) if i < c => (TEXT, Color32::TRANSPARENT),
                    Some(c) if i == c => (GROUND, ACCENT),
                    Some(_) => (FAINT, Color32::TRANSPARENT),
                };
                TextFormat { font_id: mono.clone(), color, background, ..Default::default() }
            };
            for i in from..e {
                let ch = p.chars[i];
                let mut buf = [0u8; 4];
                let txt = if ch == '\t' { "    " } else { ch.encode_utf8(&mut buf) };
                job.append(txt, 0.0, fmt(i));
            }
            if caret == Some(e) && e < p.chars.len() {
                job.append(" ", 0.0, fmt(e));
            }
        }
        if job.text.is_empty() {
            job.append(" ", 0.0, TextFormat { font_id: mono.clone(), ..Default::default() });
        }
        ui.label(job);
    }
}

fn file_name(p: &Path) -> String {
    p.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| p.display().to_string())
}

#[cfg(windows)]
fn pick_file() -> Option<PathBuf> {
    rfd::FileDialog::new().pick_file()
}

#[cfg(not(windows))]
fn pick_file() -> Option<PathBuf> {
    None
}

fn main() -> eframe::Result {
    let icon = eframe::icon_data::from_png_bytes(include_bytes!("../assets/icon.png")).unwrap_or_default();
    let opts = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(update::APP_NAME)
            .with_icon(Arc::new(icon))
            .with_inner_size([560.0, 720.0])
            .with_min_inner_size([460.0, 560.0])
            .with_drag_and_drop(true),
        ..Default::default()
    };
    eframe::run_native(update::APP_NAME, opts, Box::new(|cc| Ok(Box::new(App::new(cc)))))
}
