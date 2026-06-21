use crate::{Keyable, Symbol};
use std::fs::{File, OpenOptions};
use std::io;
use std::io::Write;
use std::path::Path;
use tokio::time::{Duration, sleep};

/// A raspberry pi activity light
pub struct ActivityLight {
    delay_on: File,
    delay_off: File,
    shot: File,
}

impl ActivityLight {
    pub fn new(kernel_dev: &Path) -> io::Result<Self> {
        let mut options = OpenOptions::new();
        options.read(false).write(true);

        let mut trigger = options.open(kernel_dev.join("trigger"))?;
        trigger.write_all(b"oneshot")?;

        println!("Set trigger");

        Ok(ActivityLight {
            delay_on: options.open(kernel_dev.join("delay_on"))?,
            delay_off: options.open(kernel_dev.join("delay_off"))?,
            shot: options.open(kernel_dev.join("shot"))?,
        })
    }

    pub fn shot(&mut self) -> io::Result<()> {
        self.shot.write_all(&[1])
    }

    pub fn set_delay(&mut self, on: Duration, off: Duration) -> io::Result<()> {
        write!(self.delay_on, "{}", on.as_millis())?;
        write!(self.delay_off, "{}", off.as_millis())?;

        Ok(())
    }
}

impl Keyable for ActivityLight {
    type Error = io::Error;

    async fn play(
        &mut self,
        on: Duration,
        off: Duration,
        _symbol: Symbol,
    ) -> Result<(), io::Error> {
        self.set_delay(on, off)?;
        self.shot()?;
        sleep(on + off).await;

        Ok(())
    }
}
