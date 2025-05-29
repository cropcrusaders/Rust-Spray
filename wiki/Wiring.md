# Wiring and Hardware

This document provides guidance on connecting common peripherals to a
Raspberry Pi based setup. Adapt the pin numbers and interfaces for your
hardware as needed.

## Required Components

- **Solenoid Valves** – e.g. TeeJet 344E
- **CAN Interface** – PiCAN3 HAT or any SocketCAN adapter
- **Flow Sensor** – DigiFlow 200 (optional)
- **GPS Receiver** – u-blox F9P or similar
- **Camera** (optional) – Pi HQ Camera v2 or USB webcam

## Example GPIO Mapping

The sample `config/Config.toml` uses the following GPIO pins for four
sprayer outputs:

| Section | GPIO Pin |
|---------|---------:|
| 1       | 23       |
| 2       | 24       |
| 3       | 25       |
| 4       | 26       |

Adjust these pins in `config/config.toml` to match your wiring.

## Power and Safety

- Ensure that valve drivers and pumps have adequate power supplies.
- Use proper fusing and emergency stop hardware.
- The software defaults to valves off if errors occur, but always verify
  fail‑safe behaviour with your specific hardware.

## Diagrams

Detailed wiring diagrams will be provided in future revisions of this
wiki. For now, refer to the table above and the comments in
`config/Config.toml` for connection hints.
