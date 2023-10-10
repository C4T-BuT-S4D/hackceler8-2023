# hackceler8-2023

Tooling for Google CTF Hackceler8 2023 by C4T BuT S4D team

## TLDR:

- [cheats-rust](cheats-rust) contains rewritten physics & parallel A* pathfinding in Rust
  (built with maturin `opt` profile)
- Hitbox highlighting, object tracing, auxiliary info display (tick, boss hp)
- Single-tick mode (press backspace to advance one tick, see player debug info in console)
- Game recordings (save everything sent to server, replay on a game laptop afterward)
- Macros (same as recordings, but simpler & controlled by UI)
- UI (default port 8888 if using `client.sh`):
    - Map screenshots
    - Recordings loading with screenshots
    - Up to 9 macros
    - Path finding parameters control, UI control
