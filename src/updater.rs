//! Checks GitHub for a newer release on launch and offers to open its page.

use std::{
    process::Command,
    sync::{
        Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
};

use objc2::MainThreadMarker;
use objc2_app_kit::{
    NSAlert, NSAlertFirstButtonReturn, NSApplication, NSControlStateValueOn, NSWorkspace,
};
use objc2_foundation::{NSString, NSURL};
use serde::Deserialize;

use crate::config::{config, persist_config};

const LATEST_RELEASE_URL: &str = "https://api.github.com/repos/margooey/Lapsus/releases/latest";

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    html_url: String,
}

/// Set once a newer release has been found, so the tick loop can cheaply check for it.
static FOUND: AtomicBool = AtomicBool::new(false);
static PENDING: Mutex<Option<Release>> = Mutex::new(None);

/// The running version. `LAPSUS_VERSION_OVERRIDE` pretends to be an older build for testing.
fn current_version() -> String {
    std::env::var("LAPSUS_VERSION_OVERRIDE").unwrap_or_else(|_| env!("CARGO_PKG_VERSION").into())
}

/// Turns "v2.2.3" into [2, 2, 3] so versions compare numerically.
fn parse_version(version: &str) -> Vec<u64> {
    version
        .trim_start_matches('v')
        .split('.')
        .map(|part| part.parse().unwrap_or(0))
        .collect()
}

fn fetch_latest_release() -> Option<Release> {
    // curl ships with macOS, which saves pulling in an HTTP client.
    let output = Command::new("curl")
        .args(["-fsSL", "--max-time", "10", LATEST_RELEASE_URL])
        .output()
        .ok()?;
    if !output.status.success() {
        log::warn!("update check failed: curl exited with {}", output.status);
        return None;
    }
    serde_json::from_slice(&output.stdout)
        .inspect_err(|err| log::warn!("update check failed: {err}"))
        .ok()
}

/// Starts the check in the background, unless the user turned it off.
pub fn check_in_background() {
    if !config().update_check_enabled {
        return;
    }
    thread::spawn(|| {
        let Some(release) = fetch_latest_release() else {
            return;
        };
        let is_newer = parse_version(&release.tag_name) > parse_version(&current_version());
        let is_skipped = config().skipped_version.as_deref() == Some(release.tag_name.as_str());
        log::debug!(
            "latest release {} (newer: {is_newer}, skipped: {is_skipped})",
            release.tag_name
        );
        if is_newer && !is_skipped {
            *PENDING.lock().unwrap() = Some(release);
            FOUND.store(true, Ordering::Release);
        }
    });
}

/// Shows the update dialog if the background check found something. Call on the main thread.
pub fn show_alert_if_found(mtm: MainThreadMarker) {
    if !FOUND.swap(false, Ordering::Acquire) {
        return;
    }
    let Some(release) = PENDING.lock().unwrap().take() else {
        return;
    };

    let alert = NSAlert::new(mtm);
    alert.setMessageText(&NSString::from_str("A new version of Lapsus is available!"));
    alert.setInformativeText(&NSString::from_str(&format!(
        "Lapsus {} is now available. You have {}.",
        release.tag_name.trim_start_matches('v'),
        current_version(),
    )));
    alert.addButtonWithTitle(&NSString::from_str("View Release"));
    alert.addButtonWithTitle(&NSString::from_str("Skip This Version"));
    alert.setShowsSuppressionButton(true);
    if let Some(checkbox) = alert.suppressionButton() {
        checkbox.setTitle(&NSString::from_str("Don't check for updates again"));
    }

    // Lapsus is a menu bar app, so bring it forward or the dialog opens behind other windows.
    NSApplication::sharedApplication(mtm).activate();
    let response = alert.runModal();

    let dont_check_again = alert
        .suppressionButton()
        .is_some_and(|checkbox| checkbox.state() == NSControlStateValueOn);

    if response == NSAlertFirstButtonReturn {
        if let Some(url) = NSURL::URLWithString(&NSString::from_str(&release.html_url)) {
            NSWorkspace::sharedWorkspace().openURL(&url);
        }
    } else {
        config().skipped_version = Some(release.tag_name);
    }
    if dont_check_again {
        config().update_check_enabled = false;
    }
    persist_config();
}
