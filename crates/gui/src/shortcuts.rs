use eframe::egui::{Key, KeyboardShortcut, Modifiers};

pub const NAV_BACK: KeyboardShortcut = KeyboardShortcut::new(Modifiers::ALT, Key::ArrowLeft);
pub const NAV_FORWARD: KeyboardShortcut = KeyboardShortcut::new(Modifiers::ALT, Key::ArrowRight);
pub const OPEN: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::O);
pub const CLOSE: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::W);
