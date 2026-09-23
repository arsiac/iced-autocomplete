//! A minimal demo of the `iced-autocomplete` widget.
//!
//! Run with: `cargo run --example basic`

use iced::widget::{column, text};
use iced::{Element, Length, Task, Theme};
use iced_autocomplete::{Filter, State, auto_complete};

fn main() -> iced::Result {
    iced::application(Demo::new, Demo::update, Demo::view)
        .theme(Demo::theme)
        .run()
}

#[derive(Debug, Clone)]
enum Message {
    Changed(String),
    Submitted(String),
    Selected(String),
}

struct Demo {
    /// The source of truth held by the application.
    state: State<&'static str>,
    last_event: String,
}

impl Demo {
    fn new() -> (Self, Task<Message>) {
        let languages: Vec<&'static str> = vec![
            "Rust",
            "Python",
            "JavaScript",
            "TypeScript",
            "Go",
            "C++",
            "Java",
            "Kotlin",
            "Swift",
            "Zig",
        ];

        (
            Self {
                // The matching rule is stated explicitly. Here: any option
                // whose name contains the query, ignoring case.
                state: State::new(
                    languages,
                    Filter::Custom(Box::new(|option: &&'static str, query: &str| {
                        option.to_lowercase().contains(&query.to_lowercase())
                    })),
                ),
                last_event: String::from("Type something…"),
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Changed(text) => {
                self.last_event = format!("changed: {text}");
            }
            Message::Submitted(text) => {
                self.last_event = format!("submitted: {text}");
            }
            Message::Selected(option) => {
                self.last_event = format!("selected: {option}");
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        column![
            text("Autocomplete demo").size(24),
            text(&self.last_event).size(14),
            auto_complete(
                &self.state,
                "Type to filter languages…",
                Message::Changed,
                Message::Submitted,
                Some(|option: &'static str| Message::Selected(option.to_string())),
            )
            .width(Length::Fixed(300.0))
            .padding(8)
        ]
        .spacing(16)
        .padding(24)
        .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}
