// SPDX-License-Identifier: GPL-3.0-only

use crate::niri::{self, Workspace};
use cosmic::iced::{Length, Subscription};
use cosmic::prelude::*;
use cosmic::widget;
use std::sync::LazyLock;
use tokio::io::{AsyncBufReadExt, BufReader};

const MAX_TITLE_CHARS: usize = 60;

static AUTOSIZE_ID: LazyLock<widget::Id> = LazyLock::new(|| widget::Id::new("niri-workspaces"));

#[derive(Default)]
pub struct Workspaces {
    core: cosmic::Core,
    workspaces: Vec<Workspace>,
    title: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Updated(Vec<Workspace>),
    Title(Option<String>),
    Focus(String),
}

fn event_stream(_: &u8) -> impl cosmic::iced::futures::Stream<Item = Message> {
    cosmic::iced::stream::channel(
        4,
        async |mut channel: cosmic::iced::futures::channel::mpsc::Sender<Message>| {
            use cosmic::iced::futures::SinkExt;
            loop {
                if let Some(list) = niri::workspaces().await {
                    let _ = channel.send(Message::Updated(list)).await;
                }
                let _ = channel.send(Message::Title(niri::focused_title().await)).await;
                if let Some(mut child) = niri::event_stream() {
                    if let Some(stdout) = child.stdout.take() {
                        let mut lines = BufReader::new(stdout).lines();
                        while let Some(event) = niri::next_event(&mut lines).await {
                            match event {
                                niri::Event::Workspaces => {
                                    if let Some(list) = niri::workspaces().await {
                                        let _ = channel.send(Message::Updated(list)).await;
                                    }
                                    let title = niri::focused_title().await;
                                    let _ = channel.send(Message::Title(title)).await;
                                }
                                niri::Event::Window => {
                                    let title = niri::focused_title().await;
                                    let _ = channel.send(Message::Title(title)).await;
                                }
                            }
                        }
                    }
                }
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
        },
    )
}

impl cosmic::Application for Workspaces {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;

    const APP_ID: &'static str = "io.github.devantic.CosmicExtAppletNiriWorkspaces";

    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    fn init(
        core: cosmic::Core,
        _flags: Self::Flags,
    ) -> (Self, Task<cosmic::Action<Self::Message>>) {
        (
            Workspaces {
                core,
                workspaces: Vec::new(),
                title: None,
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::Updated(list) => self.workspaces = list,
            Message::Title(title) => self.title = title,
            Message::Focus(reference) => niri::focus(reference),
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let horizontal = self.core.applet.is_horizontal();
        let thickness = self.core.applet.suggested_size(true).1
            + 2 * self.core.applet.suggested_padding(true).1;

        let buttons: Vec<Element<'_, Message>> = self
            .workspaces
            .iter()
            .map(|ws| {
                let class = if ws.is_focused || ws.is_active {
                    cosmic::theme::Button::Suggested
                } else {
                    cosmic::theme::Button::AppletIcon
                };
                let label = self.core.applet.text(ws.label());
                let content = widget::container(label).center(Length::Fill);
                let button = widget::button::custom(content)
                    .padding(0)
                    .class(class)
                    .on_press(Message::Focus(ws.reference()));
                if horizontal {
                    button
                        .width(Length::Fixed(f32::from(thickness) * 0.9))
                        .height(Length::Fixed(f32::from(thickness)))
                        .into()
                } else {
                    button
                        .width(Length::Fixed(f32::from(thickness)))
                        .height(Length::Fixed(f32::from(thickness) * 0.9))
                        .into()
                }
            })
            .collect();

        let content: Element<'_, Message> = if horizontal {
            let mut row = widget::row::with_children(buttons)
                .spacing(2)
                .align_y(cosmic::iced::Alignment::Center);
            if let Some(title) = &self.title {
                let shown = if title.chars().count() > MAX_TITLE_CHARS {
                    let cut: String = title.chars().take(MAX_TITLE_CHARS).collect();
                    format!("{}…", cut.trim_end())
                } else {
                    title.clone()
                };
                row = row.push(
                    widget::container(self.core.applet.text(shown)).padding([0, 0, 0, 10]),
                );
            }
            row.into()
        } else {
            widget::column::with_children(buttons).spacing(2).into()
        };

        widget::autosize::autosize(content, AUTOSIZE_ID.clone()).into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::run_with(1u8, event_stream)
    }

    fn style(&self) -> Option<cosmic::iced::theme::Style> {
        Some(cosmic::applet::style())
    }
}
