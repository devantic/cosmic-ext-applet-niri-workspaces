// SPDX-License-Identifier: GPL-3.0-only

mod app;
mod niri;

fn main() -> cosmic::iced::Result {
    cosmic::applet::run::<app::Workspaces>(())
}
