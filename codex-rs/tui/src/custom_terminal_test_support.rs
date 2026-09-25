//! Inspect the completed frame without adding test helpers to the terminal implementation.

use std::io;
use std::io::Write;

use ratatui::backend::Backend;
use ratatui::buffer::Buffer;
use ratatui::layout::Size;

use super::Terminal;

/// Change the simulated backend size separately from a terminal resize event.
pub(crate) fn set_screen_size<B>(terminal: &mut Terminal<B>, size: Size)
where
    B: Backend<Error = io::Error> + Write,
{
    terminal.screen_size_override = Some(size);
}

pub(crate) fn last_rendered_buffer<B>(terminal: &Terminal<B>) -> &Buffer
where
    B: Backend<Error = io::Error> + Write,
{
    terminal.previous_buffer()
}
