use crossterm::{
    cursor::Show,
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use std::io::stdout;

pub struct TerminalGuard {
    mouse_capture: bool,
}

impl TerminalGuard {
    pub fn new(mouse_capture: bool) -> Result<Self, std::io::Error> {
        enable_raw_mode()?;
        let mut out = stdout();
        let res = if mouse_capture {
            execute!(out, EnterAlternateScreen, EnableMouseCapture)
        } else {
            execute!(out, EnterAlternateScreen)
        };
        if let Err(e) = res {
            let _ = disable_raw_mode();
            return Err(e);
        }
        Ok(Self { mouse_capture })
    }

    #[allow(dead_code)]
    pub fn mouse_capture(&self) -> bool {
        self.mouse_capture
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let mut out = stdout();
        if self.mouse_capture {
            let _ = execute!(out, DisableMouseCapture);
        }
        let _ = execute!(out, LeaveAlternateScreen, Show);
    }
}

#[cfg(test)]
#[path = "terminal_unit.rs"]
mod tests;
