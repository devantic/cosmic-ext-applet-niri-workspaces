// SPDX-License-Identifier: GPL-3.0-only

use serde::Deserialize;
use std::process::Stdio;
use tokio::io::BufReader;
use tokio::process::Command;

#[derive(Debug, Clone, Deserialize)]
pub struct Workspace {
    pub idx: u32,
    pub name: Option<String>,
    pub output: Option<String>,
    #[allow(dead_code)]
    pub is_urgent: bool,
    pub is_active: bool,
    pub is_focused: bool,
}

impl Workspace {
    /// Reference accepted by `niri msg action focus-workspace`.
    pub fn reference(&self) -> String {
        self.name.clone().unwrap_or_else(|| self.idx.to_string())
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

/// Waits for the next workspace related event. Returns false if the stream ended.
pub async fn next_event(lines: &mut tokio::io::Lines<BufReader<tokio::process::ChildStdout>>) -> bool {
    loop {
        match lines.next_line().await {
            Ok(Some(line)) => {
                if line.starts_with("{\"WorkspacesChanged\"")
                    || line.starts_with("{\"WorkspaceActivated\"")
                    || line.starts_with("{\"WorkspaceUrgencyChanged\"")
                {
                    return true;
                }
            }
            _ => return false,
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
