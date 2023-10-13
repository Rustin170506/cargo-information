use anstyle::{AnsiColor, Effects, Style};

pub(crate) const NOP: Style = Style::new();
pub(crate) const GREEN: Style = AnsiColor::Green.on_default().effects(Effects::BOLD);
pub(crate) const YELLOW: Style = AnsiColor::Yellow.on_default().effects(Effects::BOLD);
pub(crate) const CYAN: Style = AnsiColor::Cyan.on_default().effects(Effects::BOLD);
