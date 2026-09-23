# iced-autocomplete

[![Crates.io](https://img.shields.io/crates/v/iced-autocomplete.svg)](https://crates.io/crates/iced-autocomplete)
[![Docs.rs](https://docs.rs/iced-autocomplete/badge.svg)](https://docs.rs/iced-autocomplete)
[![License](https://img.shields.io/crates/l/iced-autocomplete.svg)](#license)
[![CI](https://github.com/arsiac/iced-autocomplete/actions/workflows/ci.yml/badge.svg)](https://github.com/arsiac/iced-autocomplete/actions/workflows/ci.yml)

An autocomplete text input widget for [Iced](https://iced.rs) (0.14): a
`text_input`-like field with a suggestion menu.

Unlike `combo_box`, **the text value is the single source of truth** — the
option list only serves as hints for quick filling, and content typed outside
of the option list is submitted as a regular value. This makes the widget
suitable for both "pick from a known list" and "type anything" use cases,
including asynchronous remote search.

## Features

- Free-text submission: values outside the option list are allowed.
- Keyboard support: `↑`/`↓` (or `Shift+Tab`/`Tab`) to move, `Enter`/`Tab` to
  accept, `Escape` to dismiss.
- **Explicit filtering**: you state the matching rule; the widget ships no
  default. See [Filtering](#filtering).
- **Async-friendly**: hand a remote result set to `State::set_options` and it
  is stored verbatim.
- Themable: reuses the standard `text_input` and `menu` catalogs, with
  per-widget style overrides (`input_style`, `menu_style`).
- No iced patches: built purely on public `iced_widget` APIs.

## Filtering

There is no default matching rule: what counts as a match depends on your data
and your users' intent — case sensitivity, which fields to search, whether
initials or pinyin should count, how non-ASCII text is handled. A built-in
guess would be silently wrong for some callers, so the rule is yours to declare
through `Filter`:

```rust
use iced_autocomplete::{Filter, State};

// An in-memory pool: your own predicate, over any field you like.
let state: State<Person> = State::new(
    people,
    Filter::Custom(Box::new(|person: &Person, query: &str| {
        person.username.to_lowercase().contains(&query.to_lowercase())
    })),
);

// A remote source: results are already exact, so nothing is filtered locally.
let state: State<Item> = State::new(Vec::new(), Filter::None);
```

## Remote search

The widget performs no I/O. In iced, widgets publish messages and only your
`update` can spawn work, so a remote search is wired up in the application:

1. `on_input` publishes your own message carrying the typed text;
2. your `update` starts the request (e.g. with `Task::perform`);
3. on response, hand the results to `State::set_options`. Because the server
   already applied the matching rules, build the state with `Filter::None` —
   a second local filter would wrongly narrow the results.

Debouncing and out-of-order responses also belong to the application, since
only it can see the requests. The `remote` example demonstrates both.

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
iced = "0.14"
iced-autocomplete = "0.1"
```

```rust
use iced_autocomplete::{Filter, State, auto_complete};

#[derive(Debug, Clone)]
enum Message {
    Changed(String),
    Submitted(String),
    Selected(String),
}

fn state() -> State<String> {
    State::new(
        vec!["Rust".to_string(), "Ruby".to_string()],
        Filter::Custom(Box::new(|option: &String, query: &str| {
            option.to_lowercase().contains(&query.to_lowercase())
        })),
    )
}

fn view(state: &State<String>) -> iced::Element<'_, Message> {
    auto_complete(
        state,
        "Type to search…",
        |text| Message::Changed(text),
        |text| Message::Submitted(text),
        Some(|option| Message::Selected(option)),
    )
    .width(300)
    .into()
}
```

## License

Licensed under the [MIT license](LICENSE).
