// Updates and installation.
//
// Every release carries two builds:
// - AutoTyper-Setup.exe: the installer. Installed copies update by running it silently.
// - auto-typer.exe: the bare executable. Portable copies (and the 0.1.x updater, which
//   only knows this name) update by replacing themselves with it.
// A portable copy is also offered, once, to install itself; the installer then relaunches
// the installed copy, which offers to delete the portable one.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use eframe::egui::Context;
use serde::Deserialize;
use sha2::{Digest, Sha256};

pub const APP_NAME: &str = "AutoTyper";
const EXE_ASSET: &str = "auto-typer.exe";
const SETUP_ASSET: &str = "AutoTyper-Setup.exe";
const MAX_BYTES: u64 = 100 * 1024 * 1024;

#[derive(Clone, PartialEq)]
struct Asset {
    url: String,
    sum_url: String,
}

#[derive(Clone, PartialEq)]
pub struct Release {
    pub version: String,
    exe: Option<Asset>,
    setup: Option<Asset>,
}

#[derive(Clone, PartialEq)]
pub enum Update {
    Idle,
    /// `install` is the one-time offer to move a portable copy into an installation;
    /// otherwise it is a newer version.
    Offer { rel: Release, install: bool },
    Working(String),
    Failed { rel: Release, install: bool, msg: String },
    Restart,
}

#[derive(Deserialize)]
struct GhRelease {
    tag_name: String,
    assets: Vec<GhAsset>,
}

#[derive(Deserialize)]
struct GhAsset {
    name: String,
    browser_download_url: String,
}

pub fn current() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Where the installer puts the app: %LOCALAPPDATA%\Programs\AutoTyper.
fn install_dir() -> Option<PathBuf> {
    let base = std::env::var_os("LOCALAPPDATA")?;
    Some(PathBuf::from(base).join("Programs").join(APP_NAME))
}

fn same_path(a: &Path, b: &Path) -> bool {
    let norm = |p: &Path| p.to_string_lossy().trim_end_matches('\\').to_lowercase();
    norm(a) == norm(b)
}

pub fn is_installed() -> bool {
    let (Ok(exe), Some(dir)) = (std::env::current_exe(), install_dir()) else {
        return false;
    };
    exe.parent().is_some_and(|p| same_path(p, &dir))
}

fn offer_marker() -> Option<PathBuf> {
    let base = std::env::var_os("LOCALAPPDATA")?;
    Some(PathBuf::from(base).join(APP_NAME).join("no-install-offer"))
}

fn offer_dismissed() -> bool {
    offer_marker().is_some_and(|p| p.exists())
}

fn repo() -> Option<String> {
    let u = env!("CARGO_PKG_REPOSITORY")
        .trim_end_matches('/')
        .trim_end_matches(".git");
    let mut it = u.rsplit('/');
    let name = it.next()?;
    let owner = it.next()?;
    if name.is_empty() || owner.is_empty() || owner == "OWNER" {
        return None;
    }
    Some(format!("{owner}/{name}"))
}

fn agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(20))
        .user_agent(concat!("auto-typer/", env!("CARGO_PKG_VERSION")))
        .build()
}

/// The latest release and whether it is newer than this build.
fn latest() -> Result<Option<(Release, bool)>, String> {
    let Some(repo) = repo() else {
        return Ok(None);
    };
    let url = format!("https://api.github.com/repos/{repo}/releases/latest");
    let rel: GhRelease = agent()
        .get(&url)
        .set("Accept", "application/vnd.github+json")
        .call()
        .map_err(|e| e.to_string())?
        .into_json()
        .map_err(|e| e.to_string())?;
    let tag = rel.tag_name.trim_start_matches('v');
    let remote = semver::Version::parse(tag).map_err(|e| e.to_string())?;
    let local = semver::Version::parse(current()).map_err(|e| e.to_string())?;
    let find = |n: &str| {
        let url = rel.assets.iter().find(|a| a.name == n)?.browser_download_url.clone();
        let sum = format!("{n}.sha256");
        let sum_url = rel.assets.iter().find(|a| a.name == sum)?.browser_download_url.clone();
        Some(Asset { url, sum_url })
    };
    let out = Release {
        version: tag.to_string(),
        exe: find(EXE_ASSET),
        setup: find(SETUP_ASSET),
    };
    Ok(Some((out, remote > local)))
}

/// What to offer for a release, given how this copy is running.
fn decide(rel: Release, newer: bool) -> Update {
    let installed = is_installed();
    if !installed && rel.setup.is_some() && !offer_dismissed() {
        return Update::Offer { rel, install: true };
    }
    let can = if installed { rel.setup.is_some() } else { rel.exe.is_some() };
    if newer && can {
        Update::Offer { rel, install: false }
    } else {
        Update::Idle
    }
}

pub fn check(state: Arc<Mutex<Update>>, ctx: Context) {
    thread::spawn(move || {
        if let Ok(Some((rel, newer))) = latest() {
            *state.lock().unwrap() = decide(rel, newer);
            ctx.request_repaint();
        }
    });
}

/// "Not now" on the install offer: never offer it again, but still offer newer versions.
pub fn dismiss_install(state: &Mutex<Update>) {
    if let Some(p) = offer_marker() {
        if let Some(dir) = p.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let _ = std::fs::write(&p, b"");
    }
    let mut s = state.lock().unwrap();
    if let Update::Offer { rel, install: true } = &*s {
        let newer = semver::Version::parse(&rel.version)
            .ok()
            .zip(semver::Version::parse(current()).ok())
            .is_some_and(|(r, l)| r > l);
        *s = decide(rel.clone(), newer);
    }
}

pub fn run(rel: Release, install: bool, state: Arc<Mutex<Update>>, ctx: Context) {
    let via_setup = install || is_installed();
    let msg = if install {
        format!("Installing {APP_NAME} {}…", rel.version)
    } else {
        format!("Downloading version {}…", rel.version)
    };
    *state.lock().unwrap() = Update::Working(msg);
    thread::spawn(move || {
        let result = if via_setup { run_setup(&rel, install) } else { replace_self(&rel) };
        *state.lock().unwrap() = match result {
            Ok(()) => Update::Restart,
            Err(msg) => Update::Failed { rel, install, msg },
        };
        ctx.request_repaint();
    });
}

fn fetch(url: &str) -> Result<Vec<u8>, String> {
    let resp = agent().get(url).call().map_err(|e| e.to_string())?;
    let mut buf = Vec::new();
    resp.into_reader()
        .take(MAX_BYTES)
        .read_to_end(&mut buf)
        .map_err(|e| e.to_string())?;
    Ok(buf)
}

/// Downloads an asset and checks it against its .sha256 file.
fn fetch_verified(a: &Asset) -> Result<Vec<u8>, String> {
    let sum = String::from_utf8(fetch(&a.sum_url)?).map_err(|e| e.to_string())?;
    let expected = sum
        .split_whitespace()
        .next()
        .ok_or("checksum file is empty")?
        .to_ascii_lowercase();
    let bytes = fetch(&a.url)?;
    let actual: String = Sha256::digest(&bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    if actual != expected {
        return Err("Downloaded file failed the checksum check.".into());
    }
    Ok(bytes)
}

fn replace_self(rel: &Release) -> Result<(), String> {
    let asset = rel.exe.as_ref().ok_or("This release has no executable.")?;
    let bytes = fetch_verified(asset)?;
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let tmp: PathBuf = std::env::temp_dir().join(format!("auto-typer-{}.exe", rel.version));
    std::fs::write(&tmp, &bytes).map_err(|e| e.to_string())?;
    let replaced = self_replace::self_replace(&tmp).map_err(|e| e.to_string());
    let _ = std::fs::remove_file(&tmp);
    replaced?;
    Command::new(&exe).spawn().map_err(|e| e.to_string())?;
    Ok(())
}

/// Runs the installer silently. It closes this window, installs, and relaunches the
/// installed copy. When moving from a portable copy it passes that copy's path along
/// so the installed copy can offer to delete it.
fn run_setup(rel: &Release, migrating: bool) -> Result<(), String> {
    let asset = rel.setup.as_ref().ok_or("This release has no installer.")?;
    let bytes = fetch_verified(asset)?;
    let tmp = std::env::temp_dir().join(format!("AutoTyper-Setup-{}.exe", rel.version));
    std::fs::write(&tmp, &bytes).map_err(|e| e.to_string())?;
    let mut cmd = Command::new(&tmp);
    cmd.args(["/VERYSILENT", "/SUPPRESSMSGBOXES", "/NORESTART", "/RELAUNCH=1"]);
    if migrating {
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        cmd.arg(format!("/FROM={}", exe.display()));
    }
    cmd.spawn().map_err(|e| e.to_string())?;
    Ok(())
}

/// The portable copy this installed copy replaced, from `--migrated-from <path>`.
pub fn migrated_from() -> Option<PathBuf> {
    let mut args = std::env::args_os().skip(1);
    while let Some(a) = args.next() {
        if a == "--migrated-from" {
            let p = PathBuf::from(args.next()?);
            let me = std::env::current_exe().ok()?;
            if p.as_os_str().is_empty() || same_path(&p, &me) || !p.is_file() {
                return None;
            }
            return Some(p);
        }
    }
    None
}

/// Deletes the old portable copy. Its process may still be closing, so retry briefly.
pub fn delete_old_copy(p: &Path) -> Result<(), String> {
    let mut last = String::new();
    for _ in 0..10 {
        match std::fs::remove_file(p) {
            Ok(()) => return Ok(()),
            Err(e) => last = e.to_string(),
        }
        thread::sleep(Duration::from_millis(300));
    }
    Err(last)
}
