#![windows_subsystem = "windows"]

mod theme;
mod typer;
mod update;

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use eframe::egui::{self, Frame, Label, Margin, ProgressBar, RichText, Ui};
use theme::{ERROR, SUCCESS, TEXT, TEXT_DIM, WARNING};
use typer::{Job, Mode, Phase};
use update::Update;

const COUNTDOWN_S: u32 = 5;

enum Src {
    Empty,
    Ready { path: PathBuf, text: String },
    Failed { msg: String },
}

struct App {
    src: Src,
    delay_ms: u32,
    shift_gap_ms: u32,
    mode: Mode,
    strip: bool,
    phase: Arc<Mutex<Phase>>,
    stop: Arc<AtomicBool>,
    update: Arc<Mutex<Update>>,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::load_cjk_font(&cc.egui_ctx);
        theme::apply(&cc.egui_ctx);
        let update = Arc::new(Mutex::new(Update::Idle));
        update::check(update.clone(), cc.egui_ctx.clone());
        Self {
            src: Src::Empty,
            delay_ms: 15,
            shift_gap_ms: 0,
            mode: Mode::Keys,
            strip: false,
            phase: Arc::new(Mutex::new(Phase::Idle)),
            stop: Arc::new(AtomicBool::new(false)),
            update,
        }
    }

    fn update_banner(&mut self, ui: &mut Ui, ctx: &egui::Context, busy: bool) {
        let state = self.update.lock().unwrap().clone();
        let rel = match &state {
            Update::Idle | Update::Restart => return,
            Update::Available(r) | Update::Downloading(r) | Update::Failed(r, _) => r.clone(),
        };
        let mut go = false;
        theme::section(ui, "UPDATE", false, |ui| match &state {
            Update::Available(r) => {
                ui.label(RichText::new(format!("Version {} is available.", r.version)).color(TEXT));
                ui.label(RichText::new(format!("You are on {}.", update::current())).color(TEXT_DIM));
                ui.add_space(8.0);
                go = theme::button(ui, "Update and restart", !busy, true, 200.0).clicked();
            }
            Update::Downloading(r) => {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(RichText::new(format!("Downloading version {}…", r.version)).color(TEXT));
                });
            }
            Update::Failed(_, msg) => {
                ui.add(Label::new(RichText::new(format!("Update failed: {msg}")).color(ERROR)).wrap());
                ui.add_space(8.0);
                go = theme::button(ui, "Retry", !busy, false, 120.0).clicked();
            }
            _ => {}
        });
        ui.add_space(16.0);
        if go {
            update::install(rel, self.update.clone(), ctx.clone());
        }
    }

    fn load(&mut self, path: PathBuf) {
        self.src = match std::fs::read(&path) {
            Err(e) => Src::Failed { msg: format!("Cannot read file: {e}") },
            Ok(b) if b.is_empty() => Src::Failed { msg: "The file is empty.".into() },
            Ok(b) => match String::from_utf8(b) {
                Ok(text) => Src::Ready { path, text },
                Err(_) => Src::Failed {
                    msg: "The file is not UTF-8 text. Save it as UTF-8 and try again.".into(),
                },
            },
        };
        *self.phase.lock().unwrap() = Phase::Idle;
    }

    fn start(&mut self, ctx: &egui::Context) {
        let Src::Ready { text, .. } = &self.src else {
            return;
        };
        let job = Job {
            chars: typer::prepare(text, self.strip),
            delay_ms: self.delay_ms,
            shift_gap_ms: self.shift_gap_ms,
            mode: self.mode,
            countdown_s: COUNTDOWN_S,
        };
        self.stop.store(false, Ordering::Relaxed);
        *self.phase.lock().unwrap() = Phase::Countdown(COUNTDOWN_S);
        typer::spawn(job, self.phase.clone(), self.stop.clone(), ctx.clone());
    }

    fn file_section(&mut self, ui: &mut Ui, busy: bool, dragging: bool) {
        let mut pick = false;
        theme::section(ui, "FILE", dragging && !busy, |ui| match &self.src {
            Src::Empty => {
                ui.label(RichText::new("No file selected").size(15.0).color(TEXT));
                ui.label(RichText::new("Drag a text file onto this window, or browse.").color(TEXT_DIM));
                ui.add_space(8.0);
                pick = theme::button(ui, "Browse…", !busy, false, 120.0).clicked();
            }
            Src::Ready { path, text } => {
                theme::card(ui, |ui| {
                    ui.add(Label::new(RichText::new(file_name(path)).size(15.0).color(TEXT)).truncate())
                        .on_hover_text(path.display().to_string());
                    ui.label(
                        RichText::new(format!(
                            "{} characters · {} lines",
                            text.chars().count(),
                            text.lines().count()
                        ))
                        .color(TEXT_DIM),
                    );
                });
                ui.add_space(8.0);
                pick = theme::button(ui, "Change file…", !busy, false, 120.0).clicked();
            }
            Src::Failed { msg } => {
                ui.add(Label::new(RichText::new(msg).color(ERROR)).wrap());
                ui.add_space(8.0);
                pick = theme::button(ui, "Browse…", !busy, false, 120.0).clicked();
            }
        });
        if pick {
            if let Some(p) = pick_file() {
                self.load(p);
            }
        }
    }

    fn settings_section(&mut self, ui: &mut Ui, busy: bool) {
        theme::section(ui, "SETTINGS", false, |ui| {
            ui.add_enabled_ui(!busy, |ui| {
                ui.label(RichText::new("Delay per character").color(TEXT));
                let r = ui.add(egui::Slider::new(&mut self.delay_ms, 0..=200).suffix(" ms"));
                theme::focus_ring(ui, &r);
                ui.label(
                    RichText::new("Raise this if characters go missing in remote desktops.")
                        .size(12.0)
                        .color(TEXT_DIM),
                );
                ui.add_space(8.0);
                ui.label(RichText::new("Shift gap").color(TEXT));
                let r = ui.add_enabled(
                    self.mode == Mode::Keys,
                    egui::Slider::new(&mut self.shift_gap_ms, 0..=100).suffix(" ms"),
                );
                theme::focus_ring(ui, &r);
                ui.label(
                    RichText::new("Pause between Shift and the letter. Raise this if capital letters come out wrong over chained remote connections. 0 sends them together.")
                        .size(12.0)
                        .color(TEXT_DIM),
                );
                ui.add_space(8.0);
                ui.label(RichText::new("Input method").color(TEXT));
                let r = ui.radio_value(&mut self.mode, Mode::Keys, "Keystrokes — remote desktop, VM, terminals");
                theme::focus_ring(ui, &r);
                let r = ui.radio_value(&mut self.mode, Mode::Unicode, "Unicode — local apps, supports CJK text");
                theme::focus_ring(ui, &r);
                ui.add_space(8.0);
                let r = ui.checkbox(&mut self.strip, "Strip leading indentation (editors with auto-indent)");
                theme::focus_ring(ui, &r);
            });
        });
    }

    fn status_section(&self, ui: &mut Ui, phase: &Phase) {
        theme::section(ui, "STATUS", false, |ui| match phase {
            Phase::Idle => {
                let msg = match self.src {
                    Src::Ready { .. } => "Ready. After pressing Start, click into the target window within 5 seconds.",
                    _ => "Waiting for a file.",
                };
                ui.add(Label::new(RichText::new(msg).color(TEXT_DIM)).wrap());
            }
            Phase::Countdown(s) => {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(RichText::new(format!("Typing starts in {s} s")).size(18.0).strong().color(TEXT));
                });
                ui.label(RichText::new("Click into the target window now. Press Esc to stop.").color(TEXT_DIM));
            }
            Phase::Typing { done, total } => {
                let f = if *total == 0 { 1.0 } else { *done as f32 / *total as f32 };
                ui.add(ProgressBar::new(f).text(format!("{done} / {total}")));
                ui.label(RichText::new("Typing… Press Esc to stop.").color(TEXT_DIM));
            }
            Phase::Done(n) => {
                ui.label(RichText::new(format!("Finished: {n} characters typed.")).color(SUCCESS));
            }
            Phase::Stopped(n) => {
                ui.label(RichText::new(format!("Stopped after {n} characters.")).color(WARNING));
            }
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

        egui::CentralPanel::default()
            .frame(Frame::new().fill(theme::BG).inner_margin(Margin::same(24)))
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.label(RichText::new("Auto Typer").size(22.0).strong().color(TEXT));
                    ui.label(RichText::new("Types a text file into whichever window has focus.").color(TEXT_DIM));
                    ui.add_space(16.0);
                    self.update_banner(ui, ctx, busy);
                    self.file_section(ui, busy, dragging);
                    ui.add_space(16.0);
                    self.settings_section(ui, busy);
                    ui.add_space(16.0);
                    let w = ui.available_width();
                    if busy {
                        if theme::button(ui, "Stop", true, false, w).clicked() {
                            self.stop.store(true, Ordering::Relaxed);
                        }
                    } else {
                        let ready = matches!(self.src, Src::Ready { .. });
                        if theme::button(ui, "Start  (types after 5 s)", ready, true, w).clicked() {
                            self.start(ctx);
                        }
                    }
                    ui.add_space(16.0);
                    self.status_section(ui, &phase);
                    ui.add_space(8.0);
                    ui.add(
                        Label::new(
                            RichText::new("Tip: switch your input method to English before starting.")
                                .size(12.0)
                                .color(TEXT_DIM),
                        )
                        .wrap(),
                    );
                    ui.label(
                        RichText::new(format!("Version {}", update::current()))
                            .size(12.0)
                            .color(TEXT_DIM),
                    );
                });
            });
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
            .with_title("Auto Typer")
            .with_icon(Arc::new(icon))
            .with_inner_size([520.0, 720.0])
            .with_min_inner_size([420.0, 560.0])
            .with_drag_and_drop(true),
        ..Default::default()
    };
    eframe::run_native("Auto Typer", opts, Box::new(|cc| Ok(Box::new(App::new(cc)))))
}
