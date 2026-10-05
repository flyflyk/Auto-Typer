use std::io::Read;
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use eframe::egui::Context;
use serde::Deserialize;
use sha2::{Digest, Sha256};

const ASSET: &str = "auto-typer.exe";
const MAX_BYTES: u64 = 100 * 1024 * 1024;

#[derive(Clone, PartialEq)]
pub struct Release {
    pub version: String,
    exe_url: String,
    sum_url: String,
}

#[derive(Clone, PartialEq)]
pub enum Update {
    Idle,
    Available(Release),
    Downloading(Release),
    Failed(Release, String),
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

fn latest() -> Result<Option<Release>, String> {
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
    if remote <= local {
        return Ok(None);
    }
    let find = |n: &str| {
        rel.assets
            .iter()
            .find(|a| a.name == n)
            .map(|a| a.browser_download_url.clone())
    };
    let sum_name = format!("{ASSET}.sha256");
    match (find(ASSET), find(&sum_name)) {
        (Some(exe_url), Some(sum_url)) => Ok(Some(Release {
            version: tag.to_string(),
            exe_url,
            sum_url,
        })),
        _ => Ok(None),
    }
}

pub fn check(state: Arc<Mutex<Update>>, ctx: Context) {
    thread::spawn(move || {
        if let Ok(Some(rel)) = latest() {
            *state.lock().unwrap() = Update::Available(rel);
            ctx.request_repaint();
        }
    });
}

pub fn install(rel: Release, state: Arc<Mutex<Update>>, ctx: Context) {
    *state.lock().unwrap() = Update::Downloading(rel.clone());
    thread::spawn(move || {
        let next = match download_and_replace(&rel) {
            Ok(()) => Update::Restart,
            Err(e) => Update::Failed(rel, e),
        };
        *state.lock().unwrap() = next;
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

fn download_and_replace(rel: &Release) -> Result<(), String> {
    let sum = String::from_utf8(fetch(&rel.sum_url)?).map_err(|e| e.to_string())?;
    let expected = sum
        .split_whitespace()
        .next()
        .ok_or("checksum file is empty")?
        .to_ascii_lowercase();
    let bytes = fetch(&rel.exe_url)?;
    let actual: String = Sha256::digest(&bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    if actual != expected {
        return Err("Downloaded file failed the checksum check.".into());
    }

    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let tmp: PathBuf = std::env::temp_dir().join(format!("auto-typer-{}.exe", rel.version));
    std::fs::write(&tmp, &bytes).map_err(|e| e.to_string())?;
    let replaced = self_replace::self_replace(&tmp).map_err(|e| e.to_string());
    let _ = std::fs::remove_file(&tmp);
    replaced?;

    Command::new(&exe).spawn().map_err(|e| e.to_string())?;
    Ok(())
}
