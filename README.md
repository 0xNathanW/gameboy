# Gameboy
A Gameboy emulator written in Rust, with support for WebAssembly.

Play in the browser -> [0xnathanw.github.io/gameboy/](https://0xnathanw.github.io/gameboy/)

<img src="assets/pokemon.gif" width="400"> <img src="assets/dr_mario.gif" width="400">

## Repo Structure

core => library crate with gb system components.

gameboy      => binary crate.

web => compiles to WebAssembly for use on the browser.

## Installation
Note - Currently only tested on Windows.

If you have rust installed you can clone the repository and build from source (be sure to compile with --release flag).

Alternatively you can grab the binary from releases.

## Usage

![image](https://user-images.githubusercontent.com/86011312/196007956-21586824-334e-42b3-96c6-cc92470c6bfa.png)

Saves will write to a .sav file in the same directory as the ROM.  Likewise, to read a save make sure it is in the same directory as the ROM.

### Controls
| Input       | Key         |
| ----------- | ----------- |
| Up/Down/Left/Right   | Arrow keys  |
| A   | Z        |
| B   | X        |
| Start | Enter  |
| Select | Space |
