//! This module contains a interface for blinking morse code on an ANSI terminal.

use crate::{Keyable, Symbol};
use std::io::{self, Write};
use tokio::time::{Duration, sleep};

pub struct TextInterface<W: Write>(W);

impl TextInterface<io::Stdout> {
    pub fn stdout() -> Self {
        TextInterface(io::stdout())
    }
}

impl<W: Write> Keyable for TextInterface<W> {
    type Error = io::Error;

    async fn play(&mut self, on: Duration, off: Duration, symbol: Symbol) -> Result<(), io::Error> {
        write!(self.0, "\u{2588}")?;
        self.0.flush()?;
        sleep(on).await;
        write!(self.0, "\x08 \x08")?;
        self.0.flush()?;
        sleep(off).await;
        Ok(())
    }
}
