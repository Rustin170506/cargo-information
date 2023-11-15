use anstyle::{AnsiColor, Effects, Style};

pub const NOP: Style = Style::new();
pub const HEADER: Style = AnsiColor::Green.on_default().effects(Effects::BOLD);
pub const LITERAL: Style = AnsiColor::Cyan.on_default().effects(Effects::BOLD);
pub const ERROR: Style = AnsiColor::Red.on_default().effects(Effects::BOLD);
pub const WARN: Style = AnsiColor::Yellow.on_default().effects(Effects::BOLD);
pub const NOTE: Style = AnsiColor::Cyan.on_default().effects(Effects::BOLD);
