use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::time::Duration;

pub enum AppEvent {
    Key(KeyEvent),
    Tick,
    Resize((), ()),
}

pub fn poll_event(timeout: Duration) -> anyhow::Result<Option<AppEvent>> {
    if event::poll(timeout)? {
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => Ok(Some(AppEvent::Key(key))),
            Event::Resize(_, _) => Ok(Some(AppEvent::Resize((), ()))),
            _ => Ok(None),
        }
    } else {
        Ok(Some(AppEvent::Tick))
    }
}

pub fn should_quit(key: &KeyEvent) -> bool {
    matches!(
        (key.code, key.modifiers),
        (KeyCode::Char('q'), KeyModifiers::NONE) | (KeyCode::Char('c'), KeyModifiers::CONTROL)
    )
}
