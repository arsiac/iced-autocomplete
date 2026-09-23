//! An example of driving the autocomplete with a **remote** data source.
//!
//! Run with: `cargo run --example remote`
//!
//! The widget performs no I/O, so the application owns both ends: `on_input`
//! publishes the query, and `State::set_options` takes the results once they
//! land. The pool is stored verbatim, hence [`Filter::None`].
//!
//! Two concerns live here because only the application can see the requests:
//!
//! - **Debouncing** — each keystroke aborts the previous pending request via
//!   `Task::abortable`, so a burst of typing produces exactly one real request.
//!   A plain delay would not do: every keystroke would still reach the server,
//!   only to be discarded afterwards.
//! - **Out-of-order responses** — `latest_request` tags the newest keystroke
//!   and stale replies are dropped, since a slow earlier request can resolve
//!   after a later one.
//!
//! `Task` cannot borrow, so the request logic lives in a module-level `async
//! fn`; a real client would use `reqwest` here.

use std::cell::Cell;
use std::time::Duration;

use iced::widget::{column, text};
use iced::{Element, Length, Task, Theme};
use iced_autocomplete::{Filter, State, auto_complete};

/// Quiet period before a pending request is allowed to contact the "server".
const DEBOUNCE: Duration = Duration::from_millis(300);

/// Simulated round-trip time, so out-of-order responses are observable.
const SIMULATED_LATENCY: Duration = Duration::from_millis(200);

fn main() -> iced::Result {
    iced::application(Demo::new, Demo::update, Demo::view)
        .theme(Demo::theme)
        .run()
}

/// A remote record. `Display` is the menu label; `to_value` decides what the
/// field receives, and here that is only the `id`.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Item {
    id: u32,
    name: String,
}

impl std::fmt::Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name, self.id)
    }
}

#[derive(Debug, Clone)]
enum Message {
    /// The user typed; carries the current text plus a request tag.
    Input {
        query: String,
        request: u64,
    },
    /// The remote "response" arrived.
    Loaded {
        request: u64,
        items: Vec<Item>,
    },
    Submitted(String),
    /// Carries the whole record, not just the id — the widget hands back `T`.
    Selected(Item),
}

struct Demo {
    state: State<Item>,
    /// Tag of the most recently issued request; older responses are dropped.
    latest_request: u64,
    /// Monotonic source for request tags. A `Cell` (not an `AtomicU64`) so the
    /// counter is per-`Demo`, and the `Fn` closure below can still bump it.
    next_request: Cell<u64>,
    /// Abort handle of the request currently waiting out its debounce, if any.
    pending: Option<iced::task::Handle>,
    status: String,
}

impl Demo {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                // Starts empty; `Filter::None` because the server filters.
                state: State::new(Vec::new(), Filter::None),
                latest_request: 0,
                next_request: Cell::new(0),
                pending: None,
                status: String::from("Type to search the remote catalog…"),
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Input { query, request } => {
                self.latest_request = request;

                // Debounce: cancel whatever is still waiting, so only the most
                // recent keystroke reaches the server. Without this, typing
                // "rust" would fire four requests.
                if let Some(handle) = self.pending.take() {
                    handle.abort();
                }

                self.status = format!("typing {query:?}… (request #{request})");

                // The request starts with a quiet period, then the round trip.
                let (task, handle) = Task::perform(fetch(query), move |items| Message::Loaded {
                    request,
                    items,
                })
                .abortable();

                self.pending = Some(handle);
                task
            }
            Message::Loaded { request, items } => {
                // Generation check: keep only the newest reply, so a slow
                // earlier response cannot overwrite it.
                if request != self.latest_request {
                    self.status = format!(
                        "discarded stale response #{request} (latest is #{})",
                        self.latest_request
                    );
                    return Task::none();
                }

                self.status = format!("{} result(s) for request #{request}", items.len());
                // Must go through `State` as well, not only the app field.
                self.state.set_options(items);
                Task::none()
            }
            Message::Submitted(text) => {
                self.status = format!("submitted: {text}");
                Task::none()
            }
            Message::Selected(item) => {
                self.status = format!("selected: {} (id {})", item.name, item.id);
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        column![
            text("Remote autocomplete demo").size(24),
            text(&self.status).size(14),
            auto_complete(
                &self.state,
                "Search a remote catalog…",
                // `on_input` only sees the text; the tag is attached here so the
                // response can be matched back to this keystroke.
                move |text| {
                    let request = self.next_request.get() + 1;
                    self.next_request.set(request);
                    Message::Input {
                        query: text,
                        request,
                    }
                },
                Message::Submitted,
                Some(Message::Selected),
            )
            // Menu shows `"Name (id)"` (Display); the field receives only `id`.
            .to_value(|item: &Item| item.id.to_string())
            .width(Length::Fixed(340.0))
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

/// Simulated remote search: never borrows, so it can be driven by `Task`.
///
/// The quiet period comes first. Only a request that survived the debounce
/// gets past it; then the round trip, which is what allows out-of-order
/// completion. A real application would issue a `reqwest` call here.
async fn fetch(query: String) -> Vec<Item> {
    tokio_like_sleep(DEBOUNCE).await;
    tokio_like_sleep(SIMULATED_LATENCY).await;

    if query.trim().is_empty() {
        return Vec::new();
    }

    let catalog = CATALOG;
    let needle = query.to_lowercase();

    catalog
        .iter()
        .filter(|(_, name)| name.to_lowercase().contains(&needle))
        .map(|(id, name)| Item {
            id: *id,
            name: (*name).to_string(),
        })
        .collect()
}

/// A tiny stand-in for `tokio::time::sleep`, keeping the example dependency-free.
async fn tokio_like_sleep(duration: Duration) {
    use std::future::Future;
    use std::pin::Pin;
    use std::task::{Context, Poll};

    struct Sleep(std::time::Instant);

    impl Future for Sleep {
        type Output = ();

        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
            if std::time::Instant::now() >= self.0 {
                Poll::Ready(())
            } else {
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }

    Sleep(std::time::Instant::now() + duration).await
}

/// Stand-in for a remote catalog behind an HTTP endpoint.
static CATALOG: &[(u32, &str)] = &[
    (101, "Rust"),
    (102, "Ruby"),
    (103, "Racket"),
    (204, "Python"),
    (205, "Perl"),
    (306, "Go"),
    (307, "Gleam"),
    (408, "Java"),
    (409, "JavaScript"),
    (410, "Julia"),
    (511, "Zig"),
    (612, "Haskell"),
    (713, "Elixir"),
    (814, "OCaml"),
    (915, "Clojure"),
];
