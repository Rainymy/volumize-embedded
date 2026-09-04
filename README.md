# Volumize (Embedded)

Embedded firmware for the Volumize hardware, a rotary encoder + display for
controlling per application volume on your desktop.

This is the hardware side of [Volumize](https://github.com/Rainymy/volumize).
It runs on an ESP32-S3 and talks to the desktop server over USB serial.

## Overview

The device shows the currently selected application on a small display and
lets you scroll through apps and adjust volume with a rotary encoder and
button. All the actual audio mixing happens on the desktop server.

### Architecture overview

```
┌──────────────────┐                  ┌───────────────────┐
│   Desktop App    │                  │   Firmware        │
│   (Server)       │    USB Serial    │   (this repo)     │
│                  │ ←──────────────→ │                   │
│ • Volume Control │                  │ • Rotary + Button │
│ • System Tray    │                  │ • Display         │
└──────────────────┘                  └───────────────────┘
```

## Hardware

Built and tested on ESP32-S3. Should work on other `esp-hal` supported
chips too, minus the pin numbers.

| Peripheral     | Use                    | Pins                      |
| -------------- | ---------------------- | ------------------------- |
| Rotary encoder | scroll / navigate      | GPIO36 (DT), GPIO40 (CLK) |
| Button         | select                 | GPIO35                    |
| I2C0           | display                | GPIO4 (SCL), GPIO5 (SDA)  |
| USB0           | CDC-ACM serial to host | GPIO19/20                 |

## Tech stack

- **Firmware**: Rust, `esp-hal`, `embassy`, `esp-rtos`
- **Transport**: USB CDC-ACM serial
- **Encoding**: CBOR (`ciborium`)

## How it works

Everything is interrupt driven. Rotary and button reads come from GPIO
interrupts, USB I/O runs in its own dedicated task, and the main loop just
polls all of it each tick and renders the display. No blocking calls, no
busy waits.

Messages to and from the host don't go directly into the main loop either.
They pass through two small queues, so a slow host or a slow display refresh
never stalls input handling.

The UI itself is just a stack of screens. Selecting something pushes a new
screen on top, going back pops it off.

## Wire protocol

Messages are CBOR encoded and framed with a 2 byte length prefix (u16,
little endian), since the embedded side reads and writes everything as raw
`u8`:

```
[ len: u8 u8 (u16 LE) ][ CBOR bytes ]
```

```rust
enum Envelope {
    Command(Command),
    Response(Response),
    Event(UpdateChange),
}
```

- `Command` / `Response`: request and reply, initiated by either side
- `Event`: unsolicited push from the host, e.g. volume changed externally

On boot, the firmware waits for a handshake with the host, then sends
`Command::GetPlaybackDevices` to bootstrap its state.

Encoding/decoding lives in `RawFrame`:

```rust
RawFrame::encode(&envelope).build()   // -> Vec<u8>, ready to write to USB
RawFrame::decode(&bytes)              // -> Envelope
```

## Quick start

### Prerequisites

- Rust (targeting the ESP32-S3, e.g. via `espup`)
- `espflash`

```bash
cargo build
cargo run   # flash + monitor
```

Logs are `defmt` over the flasher's serial monitor.

## Project status

Part of the [Volumize](https://github.com/Rainymy/volumize) project, see
its tracker for overall progress.
