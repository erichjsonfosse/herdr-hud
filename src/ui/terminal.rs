use crossterm::{
    cursor::Show,
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use std::io::stdout;
use std::panic::{self, PanicHookInfo};
use std::sync::Arc;

pub fn restore_terminal(mouse_capture: bool) {
    let _ = disable_raw_mode();
    let mut out = stdout();
    if mouse_capture {
        let _ = execute!(out, DisableMouseCapture);
    }
    let _ = execute!(out, LeaveAlternateScreen, Show);
}

type PanicHook = Box<dyn Fn(&PanicHookInfo<'_>) + Send + Sync + 'static>;

pub struct TerminalGuard {
    mouse_capture: bool,
    previous_hook: Option<Arc<PanicHook>>,
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

        let prev_hook = Arc::new(panic::take_hook());
        let hook_clone = Arc::clone(&prev_hook);
        panic::set_hook(Box::new(move |info| {
            restore_terminal(mouse_capture);
            hook_clone(info);
        }));

        Ok(Self {
            mouse_capture,
            previous_hook: Some(prev_hook),
        })
    }

    #[allow(dead_code)]
    pub fn mouse_capture(&self) -> bool {
        self.mouse_capture
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        restore_terminal(self.mouse_capture);
        if let Some(prev) = self.previous_hook.take() {
            let _ = panic::take_hook();
            panic::set_hook(Box::new(move |info| prev(info)));
        }
    }
}

#[cfg(test)]
#[path = "terminal_unit.rs"]
mod tests;
