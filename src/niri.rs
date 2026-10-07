// SPDX-License-Identifier: GPL-3.0-only

use serde::Deserialize;
use std::process::Stdio;
use tokio::io::BufReader;
use tokio::process::Command;

#[derive(Debug, Clone, Deserialize)]
pub struct Workspace {
    pub idx: u32,
    #[allow(dead_code)]
    pub name: Option<String>,
    pub output: Option<String>,
    #[allow(dead_code)]
    pub is_urgent: bool,
    pub is_active: bool,
    pub is_focused: bool,
}

impl Workspace {
    /// Reference accepted by `niri msg action focus-workspace`.
    ///
    /// Always the index: niri parses an all-numeric reference as an index, so a
    /// workspace named e.g. "1080" would otherwise focus the wrong workspace.
    pub fn reference(&self) -> String {
        self.idx.to_string()
    }

    pub fn label(&self) -> String {
        self.idx.to_string()
    }
}

/// Lists workspaces on the output that currently holds focus, ordered by index.
pub async fn workspaces() -> Option<Vec<Workspace>> {
    let out = Command::new("niri")
        .args(["msg", "--json", "workspaces"])
        .output()
        .await
        .ok()?;
    let mut all: Vec<Workspace> = serde_json::from_slice(&out.stdout).ok()?;
    let output = all
        .iter()
        .find(|w| w.is_focused)
        .or_else(|| all.iter().find(|w| w.is_active))
        .and_then(|w| w.output.clone());
    if let Some(output) = output {
        all.retain(|w| w.output.as_deref() == Some(output.as_str()));
    }
    all.sort_by_key(|w| w.idx);
    Some(all)
}

pub fn focus(reference: String) {
    std::thread::spawn(move || {
        let _ = std::process::Command::new("niri")
            .args(["msg", "action", "focus-workspace", &reference])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    });
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    Workspaces,
    Window,
}

/// Title of the focused window, if any.
pub async fn focused_title() -> Option<String> {
    let out = Command::new("niri")
        .args(["msg", "--json", "focused-window"])
        .output()
        .await
        .ok()?;
    let value: serde_json::Value = serde_json::from_slice(&out.stdout).ok()?;
    let title = value.get("title")?.as_str()?.trim().to_string();
    (!title.is_empty()).then_some(title)
}

/// Waits for the next relevant event. Returns None if the stream ended.
pub async fn next_event(
    lines: &mut tokio::io::Lines<BufReader<tokio::process::ChildStdout>>,
) -> Option<Event> {
    loop {
        let line = lines.next_line().await.ok()??;
        if line.starts_with("{\"WorkspacesChanged\"")
            || line.starts_with("{\"WorkspaceActivated\"")
            || line.starts_with("{\"WorkspaceUrgencyChanged\"")
        {
            return Some(Event::Workspaces);
        }
        if line.starts_with("{\"WindowFocusChanged\"")
            || line.starts_with("{\"WindowOpenedOrChanged\"")
            || line.starts_with("{\"WindowClosed\"")
        {
            return Some(Event::Window);
        }
    }
}

pub fn event_stream() -> Option<tokio::process::Child> {
    Command::new("niri")
        .args(["msg", "--json", "event-stream"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .ok()
}
