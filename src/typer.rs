use std::mem::size_of;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use eframe::egui::Context;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, GetKeyState, MapVirtualKeyW, SendInput, VkKeyScanW, INPUT, INPUT_0,
    INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, KEYEVENTF_SCANCODE, KEYEVENTF_UNICODE,
    MAPVK_VK_TO_VSC, VK_CAPITAL, VK_ESCAPE, VK_RETURN, VK_SHIFT, VK_TAB,
};

#[derive(Clone, Copy, PartialEq)]
pub enum Mode {
    Keys,
    Unicode,
}

#[derive(Clone, PartialEq)]
pub enum Phase {
    Idle,
    Countdown(u32),
    Typing { done: usize, total: usize },
    Done(usize),
    Stopped(usize),
}

impl Phase {
    pub fn busy(&self) -> bool {
        matches!(self, Phase::Countdown(_) | Phase::Typing { .. })
    }
}

pub struct Job {
    pub chars: Vec<char>,
    pub delay_ms: u32,
    pub shift_gap_ms: u32,
    pub mode: Mode,
    pub countdown_s: u32,
}

pub fn prepare(text: &str, strip: bool) -> Vec<char> {
    let t = text.strip_prefix('\u{feff}').unwrap_or(text).replace("\r\n", "\n");
    if !strip {
        return t.chars().collect();
    }
    t.split('\n')
        .map(|l| l.trim_start_matches([' ', '\t']))
        .collect::<Vec<_>>()
        .join("\n")
        .chars()
        .collect()
}

pub fn spawn(job: Job, phase: Arc<Mutex<Phase>>, stop: Arc<AtomicBool>, ctx: Context) {
    thread::spawn(move || {
        let set = |p: Phase| {
            *phase.lock().unwrap() = p;
            ctx.request_repaint();
        };
        let halt = || stop.load(Ordering::Relaxed) || esc_down();

        for s in (1..=job.countdown_s).rev() {
            set(Phase::Countdown(s));
            for _ in 0..10 {
                thread::sleep(Duration::from_millis(100));
                if halt() {
                    set(Phase::Stopped(0));
                    return;
                }
            }
        }

        let total = job.chars.len();
        let delay = Duration::from_millis(job.delay_ms as u64);
        let shift_gap = Duration::from_millis(job.shift_gap_ms as u64);
        let mut last = Instant::now();
        set(Phase::Typing { done: 0, total });
        for (i, &c) in job.chars.iter().enumerate() {
            if halt() {
                set(Phase::Stopped(i));
                return;
            }
            send_char(c, job.mode, shift_gap);
            if last.elapsed() >= Duration::from_millis(50) {
                set(Phase::Typing { done: i + 1, total });
                last = Instant::now();
            }
            if !delay.is_zero() {
                thread::sleep(delay);
            }
        }
        set(Phase::Done(total));
    });
}

fn esc_down() -> bool {
    // SAFETY: GetAsyncKeyState only reads global key state.
    unsafe { (GetAsyncKeyState(VK_ESCAPE as i32) as u16) & 0x8000 != 0 }
}

fn caps_on() -> bool {
    // SAFETY: GetKeyState only reads key toggle state.
    unsafe { GetKeyState(VK_CAPITAL as i32) & 1 != 0 }
}

fn send_char(c: char, mode: Mode, shift_gap: Duration) {
    match c {
        '\n' => tap_vk(VK_RETURN),
        '\t' => tap_vk(VK_TAB),
        c if c.is_control() => {}
        c if mode == Mode::Keys && tap_keys(c, shift_gap) => {}
        c => tap_unicode(c),
    }
}

fn tap_keys(c: char, shift_gap: Duration) -> bool {
    if !c.is_ascii() {
        return false;
    }
    // SAFETY: VkKeyScanW only queries the active keyboard layout.
    let r = unsafe { VkKeyScanW(c as u16) };
    if r == -1 {
        return false;
    }
    let vk = (r as u16) & 0xff;
    let mods = ((r as u16) >> 8) & 0xff;
    if mods & !1 != 0 {
        return false;
    }
    let mut shift = mods & 1 != 0;
    if c.is_ascii_alphabetic() && caps_on() {
        shift = !shift;
    }

    if shift && !shift_gap.is_zero() {
        // Send Shift, the key and the Shift release as separate events with a pause
        // between them, so slow remote links cannot reorder or squash them.
        send(&[key(VK_SHIFT, false)]);
        thread::sleep(shift_gap);
        send(&[key(vk, false), key(vk, true)]);
        thread::sleep(shift_gap);
        send(&[key(VK_SHIFT, true)]);
        return true;
    }

    let mut seq = Vec::with_capacity(4);
    if shift {
        seq.push(key(VK_SHIFT, false));
    }
    seq.push(key(vk, false));
    seq.push(key(vk, true));
    if shift {
        seq.push(key(VK_SHIFT, true));
    }
    send(&seq);
    true
}

fn tap_vk(vk: u16) {
    send(&[key(vk, false), key(vk, true)]);
}

fn tap_unicode(c: char) {
    let mut buf = [0u16; 2];
    let seq: Vec<INPUT> = c
        .encode_utf16(&mut buf)
        .iter()
        .flat_map(|&u| [uni(u, false), uni(u, true)])
        .collect();
    send(&seq);
}

fn key(vk: u16, up: bool) -> INPUT {
    // SAFETY: MapVirtualKeyW only queries the active keyboard layout.
    let scan = unsafe { MapVirtualKeyW(vk as u32, MAPVK_VK_TO_VSC) } as u16;
    let flags = KEYEVENTF_SCANCODE | if up { KEYEVENTF_KEYUP } else { 0 };
    kbd(vk, scan, flags)
}

fn uni(u: u16, up: bool) -> INPUT {
    let flags = KEYEVENTF_UNICODE | if up { KEYEVENTF_KEYUP } else { 0 };
    kbd(0, u, flags)
}

fn kbd(vk: u16, scan: u16, flags: u32) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: scan,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

fn send(seq: &[INPUT]) {
    // SAFETY: seq is a valid slice of initialized INPUT structs for the duration of the call.
    unsafe {
        SendInput(seq.len() as u32, seq.as_ptr(), size_of::<INPUT>() as i32);
    }
}
