## NES Emulator

This is just a fun project for me to learn rust.

I'm mostly following the guide [here](https://bugzmanov.github.io/nes_ebook/chapter_1.html) but making my own modifications as I go.

### Snake Game

Changing the loading program to start at 0x0600 instead of 0x0800 will run a snake game to test the cpu.

It works now!

### SDL2

https://github.com/Rust-SDL2/rust-sdl2#windows-mingw

### Getting debugger working in Clion

Had to use MinGW toolchain to get it to work.