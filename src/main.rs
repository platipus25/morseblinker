use std::fs::File;
use std::io;
use std::io::Write;
use std::path::Path;
use tokio::time::{sleep, Duration};

///
/// From TM 11-459:
/// (1) The dit is the unit of length.
/// (2) The dah is equal to three dits.
/// (3) The space between the dits and dahs within the character is equal to one dit.
/// (4) The space between characters is equal to three dits (applies only to code transmitted at a speed of 20 gpm or a
/// (5) The space between groups is equal to seven dits (applies only to code transmitted at a speed of 20 gpm or)

#[derive(Debug, Clone, Copy)]
pub enum Symbol {
    /// a dit followed by a dit space
    Dit,
    /// 3 dits followed by a dit space
    Dah,
    /// 2 dit spaces, use 3x for word spaces
    Space,
}

pub struct Keyer {
    pub dit_length: Duration,
    pub space_dit_length: Duration,
    pub light: ActivityLight,
}

impl Keyer {
    pub async fn run(&mut self, sequence: Sequence) -> io::Result<()> {
        for symbol in sequence.0 {
            match symbol {
                Symbol::Dit => {
                    self.light
                        .set_delay(self.dit_length, self.space_dit_length)?;
                    self.light.shot()?;
                    sleep(self.dit_length + self.space_dit_length).await
                }
                Symbol::Dah => {
                    self.light
                        .set_delay(3 * self.dit_length, self.space_dit_length)?;
                    self.light.shot()?;
                    sleep(3 * self.dit_length + self.space_dit_length).await
                }
                Symbol::Space => sleep(2 * self.space_dit_length).await,
            }
        }

        Ok(())
    }
}

pub struct Sequence(Vec<Symbol>);

impl Sequence {
    pub fn new(sequence: Vec<Symbol>) -> Self {
        Self(sequence)
    }
}

impl TryFrom<&str> for Sequence {
    type Error = MorseAlphabetError;

    fn try_from(other: &str) -> Result<Self, MorseAlphabetError> {
        let mut seq = Vec::new();
        for character in other.chars() {
            seq.extend_from_slice(convert_character(character)?);
            seq.push(Symbol::Space);
        }

        Ok(Sequence(seq))
    }
}

fn convert_character(character: char) -> Result<&'static [Symbol], MorseAlphabetError> {
    Ok(match character {
        'e' => &[Symbol::Dit],
        'm' => &[Symbol::Dah, Symbol::Dah],
        ' ' => &[Symbol::Space, Symbol::Space],
        _ => Err(MorseAlphabetError::UnknownCharacter)?,
    })
}

#[derive(Debug)]
pub enum MorseAlphabetError {
    UnknownCharacter,
}

/// A raspberry pi activity light
struct ActivityLight {
    delay_on: File,
    delay_off: File,
    shot: File,
}

impl ActivityLight {
    fn new(kernel_dev: &Path) -> io::Result<Self> {
        let mut trigger = File::open(kernel_dev.join("trigger"))?;
        trigger.write_all(b"oneshot")?;

        Ok(ActivityLight {
            delay_on: File::open(kernel_dev.join("delay_on"))?,
            delay_off: File::open(kernel_dev.join("delay_off"))?,
            shot: File::open(kernel_dev.join("shot"))?,
        })
    }

    fn shot(&mut self) -> io::Result<()> {
        self.shot.write_all(&[1])
    }

    fn set_delay(&mut self, on: Duration, off: Duration) -> io::Result<()> {
        write!(self.delay_on, "{}", on.as_millis())?;
        write!(self.delay_off, "{}", off.as_millis())?;

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello, world!");

    let light = ActivityLight::new(Path::new("/sys/class/leds/ACT/"))
        .expect("could not connect to light sys class device");
    let mut keyer = Keyer {
        dit_length: Duration::from_millis(60),
        space_dit_length: Duration::from_millis(60),
        light,
    };

    let message = Sequence::try_from("em me").unwrap();

    keyer.run(message).await?;

    Ok(())
}
