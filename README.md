<p align="center">
  <img src="https://github.com/cbeust/sixty.rs/assets/92322/583fc44c-ca6c-4a27-a272-4c0febdada18" width="400"/>
</p>

<p align="center">
  <h1 align="center">
    An Apple // emulator in Rust
  </h1>
</p>


<p align="center">
  <img src="https://github.com/cbeust/sixty.rs/assets/92322/df1edb10-ac16-481b-8954-185fc713066a" width="400%">
</p>



<p>
  
</p>

# Introduction

Maple // is an Apple ][ emulator written in Rust. It was initially inspired by [my Kotlin Apple \]\[ emulator](https://github.com/cbeust/sixty) but
diverged quite a bit after a while and by now, it supports a lot of additional features (double hi resolution, write, etc...).

Maple // has a specific focus on being developer friendly by exposing a lot of internal details on the emulation, e.g.

- A convenient file picker view that lets you quickly insert disks in the drives of your choice
- A Nibble view that shows you the raw nibbles contained on the current track
- A track map showing you which tracks are standard (green dot) and non standard (yellow dot)
- A disk view, so you can visualize the head as it moves across the disk
- A debugger (very much a work in progress)

## Building and running

```
$ cd apple2
$ cargo run -r
```

## Documentation

When you launch Maple // for the first time, you will have to select a folder that contains Apple ][ disk images
(.woz and .dsk). This directory will be read recursively. After this, select the disks of your choice, place them in
the drive 1 or 2, and press the `Reboot` button.

## Tests

`cargo test` will run [Klaus' functional suite for the 6502](https://github.com/Klaus2m5/6502_65C02_functional_tests), which guarantees that the emulation is "mostly"
correct. Additionally, my Kotlin emulator boots a few Apple ][ games that use precise cycle timing for their protection,
so I'm reasonably confident the cycle counting is correct as well, including the handling of page crossing and
"branch taken", but there are no tests for cycle counting.

This code is pretty rigid right now, it needs to add some kind of listener support for the memory reads and
writes in order to be usable in an emulator, but this should be pretty trivial to add.
  
## Harte's processor tests

The `harte/` directory contains a TUI runner for Tom Harte's 6502 tests:

```
cd harte
cargo run --release
```

https://github.com/cbeust/maple-2/assets/92322/222a2172-738c-4e7e-b018-d87efbc360d0

## Gallery

<p align="center">
  <img src="https://github.com/cbeust/maple-2/assets/92322/019a2eb5-e526-4825-a14c-843cd1634aa1">
  <br>
  <b>Text mode (40 columns)</b>
</p>



## Resources

- [AppleWin instructions](https://github.com/AppleWin/AppleWin/blob/master/source/CPU/cpu_instructions.inl)

