use anstyle::{AnsiColor, Effects, Style};

pub(crate) const YELLOW: Style = AnsiColor::Yellow.on_default().effects(Effects::BOLD);
pub(crate) const CYAN: Style = AnsiColor::Cyan.on_default().effects(Effects::BOLD);
