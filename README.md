# Gameboy
A Gameboy emulator written in Rust, with support for WebAssembly.

<img src="assets/pokemon.gif" width="400"> <img src="assets/dr_mario.gif" width="400">

## Web

Use the emulator through the browser at: [0xnathanw.github.io/gameboy/](https://0xnathanw.github.io/gameboy/)

<img width="2050" height="1074" alt="image" src="https://github.com/user-attachments/assets/9e487e72-ecba-441b-a9b8-b38af0f79f36" />

## Native

### Installation

If you have rust installed you can clone the repository and build from source (be sure to compile with --release flag).

Alternatively you can grab the binary from releases.

### Usage

```bash
Usage: gameboy [OPTIONS] <PATH>

Arguments:
  <PATH>  Path to rom

Options:
  -x, --scale <SCALE>    Display scale factor [default: x4] [possible values: x1, x2, x4, x8, x16]
  -a, --audio            Enable audio
  -s, --serial           Print serial write to stdout
      --save <SAVE>      Path to save file (default: <rom>.sav)
      --rtc <RTC>        Path to RTC file (default: <rom>.rtc)
  -v, --volume <VOLUME>  Audio volume (0-100) [default: 100]
  -h, --help             Print help
```

Saves will write to a .sav file in the same directory as the ROM.

### Controls
| Input       | Key         |
| ----------- | ----------- |
| Up/Down/Left/Right   | Arrow keys  |
| A   | Z        |
| B   | X        |
| Start | Enter  |
| Select | Space |
