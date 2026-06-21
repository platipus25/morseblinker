use crate::activity_light::ActivityLight;
use crate::text_interface::TextInterface;
use std::fmt;
use std::io;
use std::path::Path;
use tokio::time::Duration;

mod activity_light;
mod text_interface;

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

    CharacterSpace,
    /// 2 dit spaces, use 3x for word spaces
    GroupSpace,
}

impl fmt::Display for Symbol {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(
            fmt,
            "{}",
            match self {
                Symbol::Dit => ".",
                Symbol::Dah => "-",
                Symbol::CharacterSpace => " ",
                Symbol::GroupSpace => " / ",
            }
        )
    }
}

pub struct Keyer<K: Keyable> {
    pub dit_length: Duration,
    pub space_dit_length: Duration,
    pub transport: K,
}

impl<K: Keyable> Keyer<K>
where
    io::Error: From<<K as Keyable>::Error>,
{
    pub async fn run(&mut self, sequence: Sequence) -> io::Result<()> {
        for symbol in sequence.0 {
            match symbol {
                Symbol::Dit => {
                    self.transport
                        .play(self.dit_length, self.space_dit_length, symbol)
                        .await?
                }
                Symbol::Dah => {
                    self.transport
                        .play(3 * self.dit_length, self.space_dit_length, symbol)
                        .await?
                }
                Symbol::GroupSpace => {
                    self.transport
                        .play(Duration::from_millis(0), 6 * self.space_dit_length, symbol)
                        .await?
                }
                Symbol::CharacterSpace => {
                    self.transport
                        .play(Duration::from_millis(0), 2 * self.space_dit_length, symbol)
                        .await?
                }
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

impl fmt::Display for Sequence {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        for sym in self.0.iter() {
            write!(fmt, "{}", sym)?
        }

        Ok(())
    }
}

impl TryFrom<&str> for Sequence {
    type Error = MorseAlphabetError;

    fn try_from(other: &str) -> Result<Self, MorseAlphabetError> {
        let mut seq = Vec::new();
        for character in other.chars() {
            seq.extend_from_slice(convert_character(character)?);
            seq.push(Symbol::CharacterSpace);
        }

        Ok(Sequence(seq))
    }
}

fn convert_character(character: char) -> Result<&'static [Symbol], MorseAlphabetError> {
    Ok(match character {
        'e' => &[Symbol::Dit],
        'm' => &[Symbol::Dah, Symbol::Dah],
        ' ' => &[Symbol::GroupSpace],
        _ => Err(MorseAlphabetError::UnknownCharacter)?,
    })
}

#[derive(Debug)]
pub enum MorseAlphabetError {
    UnknownCharacter,
}

pub trait Keyable {
    type Error;

    async fn play(
        &mut self,
        on: Duration,
        off: Duration,
        symbol: Symbol,
    ) -> Result<(), Self::Error>;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello, world!");

    let transport = ActivityLight::new(Path::new("/sys/class/leds/ACT/"))
        .expect("could not connect to light sys class device");
    // let transport = TextInterface::stdout();
    let mut keyer = Keyer {
        dit_length: Duration::from_millis(60),
        space_dit_length: Duration::from_millis(60),
        transport,
    };

    let message = Sequence::try_from("em mm me").unwrap();

    println!("{}", message);

    keyer.run(message).await?;

    Ok(())
}

#[tokio::test]
async fn text_output_test() {
    let transport = TextInterface::stdout();
    let mut keyer = Keyer {
        dit_length: Duration::from_millis(60),
        space_dit_length: Duration::from_millis(60),
        transport,
    };

    let message = Sequence::try_from("em mm me").unwrap();

    println!("{}", message);

    keyer.run(message).await.unwrap();
}
