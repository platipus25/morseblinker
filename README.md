# Morse Blinker

`morseblinker` is a Rust library and application that translates text into Morse code sequences and keys them out over various transports (LED, terminal, or WAV file).

## Features

- **Text-to-Morse**: Converts alphanumeric text to standard Morse timing sequences.
- **Asynchronous Keying**: Built on `tokio` for non-blocking timing.
- **Multiple Transports**:
  - 💡 **Linux LED (`ActivityLight`)**: Blinks hardware LEDs (e.g. Raspberry Pi ACT LED).
  - 🖥️ **ANSI Terminal (`TextInterface`)**: Displays live-blinking block characters in the terminal.
  - 🎵 **WAV Sound (`WavWriter`)**: Generates synthesized `.wav` audio files (requires `wav` feature).


## Quick Example (Terminal Output)

```rust
use std::time::Duration;
use morseblinker::{Keyer, Sequence, text_interface::TextInterface};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut keyer = Keyer {
        dit_length: Duration::from_millis(100),
        space_dit_length: Duration::from_millis(100),
        transport: TextInterface::stdout(),
    };

    let message = Sequence::try_from("hello world")?;
    keyer.run(message).await?;

    Ok(())
}
```

---

## Extending with Custom Transports

Implement the `Keyable` trait to support other hardware or outputs:

```rust
use std::time::Duration;
use morseblinker::{Keyable, Symbol};

pub struct CustomBlinker;

impl Keyable for CustomBlinker {
    type Error = std::io::Error;

    async fn play(&mut self, on: Duration, off: Duration, symbol: Symbol) -> Result<(), Self::Error> {
        // Toggle your hardware output here
        Ok(())
    }
}
```
