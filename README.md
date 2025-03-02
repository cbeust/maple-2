<p align="center">
  <img src="pics/logo.png" width="400"/>
</p>


<p align="center">
  <h1 align="center">
    An Apple // emulator in Rust
  </h1>
</p>

<p align="center">
  <img src="https://github.com/user-attachments/assets/95c7d36a-10b4-4dc8-8090-9cb9bad9e345">
</p>

# Introduction

Maple // is an Apple ][ emulator written in Rust. It was initially inspired by [my Kotlin Apple \]\[ emulator](https://github.com/cbeust/sixty) but
diverged quite a bit after a while and by now, it supports a lot of additional features (double hi resolution, write, etc...).

Maple // has a specific focus on being developer friendly by exposing a lot of internal details on the emulation, e.g.

- A convenient file picker view that lets you quickly insert disks in the drives of your choice
- Support for disk formats (`dsk`, `woz`) and hard drive / SmartPort (`hdv`). Try the Total Replay image!
- A Nibble view that shows you the raw nibbles contained on the current track
- A track map showing you which tracks are standard (green dot) and non standard (red dot)
- A disk view, so you can visualize the head as it moves across the disk
- A debugger

## Building and running

```
$ cargo run -r
```

## Protected disks

As of this writing, Maple-2 boots the following protected `woz` disks:

- Basic protections
  - Bouncing Kamungas
  - Commando
  - Planetfall
  - Rescue Raiders
  - Sammy Lightfoot
- Cross track sync
  - Blazing Paddles
  - Take 1
  - Hard Hat Mack
  - Marble Madness
- Half Tracks
  - The Bilestoad
- More advanced protections
  - Dino Eggs
  - Crisis Mountain
  - Miner 2049er II
- Fake bits
  - The Print Shop Companion
- Data latch protection
  - First Math Adventures - Understanding Word Problems
- Bit timing
  - Border Zone
- Spiradisc and quarter tracks
  - Frogger
  - Craze Maze Construction Set
- Ultimate protection
  - Prince of Persia (E7 + RWTS18)

## Gallery
<table>
    <tr>
        <td><img src="pics/text-40-columns.png"/></td>
        <td><img src="pics/text-80-columns.png"/></td>
    </tr>
    <tr align="center">
        <td><b>Text (40 columns)</b></td>
        <td><b>Text (80 columns)</b></td>
    </tr>
    <tr>
        <td><img src="pics/kings-quest.png"/></td>
        <td><img src="pics/airheart.png"/></td>
    </tr>
    <tr align="center">
        <td><b>King's Quest (DHGR)</b></td>
        <td><b>Airheart (DHGR)</b></td>
    </tr>
    <tr>
        <td><img src="pics/disk-view.png"/></td>
        <td><img src="pics/debugger-view.png"/></td>
    </tr>
    <tr align="center">
        <td><b>Disks view</b></td>
        <td><b>Debugger view</b></td>
    </tr>
    <tr>
        <td><img src="pics/nibbles-view.png"/></td>
        <td><img src="pics/drive-view.png"/></td>
    </tr>
    <tr align="center">
        <td><b>Nibbles view</b></td>
        <td><b>Drive view</b></td>
    </tr>
    <tr>
        <td><img src="pics/total-replay.png"/></td>
        <td><img src="pics/prince-of-persia.png"/></td>
    </tr>
    <tr align="center">
        <td><b>Total Replay (hard drive)</b></td>
        <td><b>Prince of Persia</b></td>
    </tr>
    <tr>
        <td><img src="pics/aztec.png"/></td>
        <td><img src="pics/apple-galaxians.png"/></td>
    </tr>
    <tr align="center">
        <td><b>Aztec</b></td>
        <td><b>Apple Galaxian</b></td>
    </tr>
    <tr>
        <td><img src="pics/black-cauldron.png"/></td>
        <td><img src="pics/bouncing-kamungas.png"/></td>
    </tr>
    <tr align="center">
        <td><b>The Black Cauldron</b></td>
        <td><b>Bouncing Kamungas</b></td>
    </tr>
    <tr>
        <td><img src="pics/conan.png"/></td>
        <td><img src="pics/ultima-5.png"/></td>
    </tr>
    <tr align="center">
        <td><b>Conan</b></td>
        <td><b>Ultima V</b></td>
    </tr>
</table>



## Documentation

When you launch `Maple //` for the first time, you will have to select a folder that contains Apple ][ disk images
(.woz and .dsk). This directory will be read recursively. After this, select the disks of your choice, place them in
the drive 1 or 2, and press the `Reboot` button.

### Views

#### Disks

Once you have selected a directory, all the disks found in that directory and below will be displayed in that window.
Just press "Drive 1" or "Drive 2" to insert that disk in the drive. You can use the filtering box on the side
to narrow down the disk you're looking for.

#### Nibbles

This view shows you what was found on the disk. It's made of two parts:

- At the top, the map of all the tracks (quarter tracks really) shown either in green (standard track) or yellow
  (non-standard track, most likely protected). A "." indicates an empty track (will return random bits). You
  can click on any of these tracks to take a look at its nibbles
- Below is the actual buffer of bits, corrected to show you nibbles. `Maple //` will attempt to locate the
  markers for address and data in order to facilitate identifying where these tracks start. This will only
  produce highlighted results for standard markers (`D5 AA 96` / `DE AA` and `D5 AA AD`/`DE AA`).

### Drive

THis view shows you a graphical representation of the travels of the disk drive. It is updated in real time as
the disk drive reads more tracks.

It's a great way to visualize copy protections in action. For a regular, unprotected disk, you would expect to
only see white dots. As soon as you see orange (half tracks) or red (quarter tracks) show up on this graph, it
means you are dealing with a protected disk.

## Klaus Gunctional Tests

`cargo test` will run [Klaus' functional suite for the 6502](https://github.com/Klaus2m5/6502_65C02_functional_tests), which guarantees that the emulation is "mostly"
correct.

## Harte's processor tests

The `harte/` directory contains a TUI runner for Tom Harte's 6502 tests:

```
cd harte
cargo run --release
```

https://github.com/cbeust/sixty.rs/assets/92322/db46d0e5-5c16-4dad-8a27-4290a41151dc

## Developer note

I made the graphical back-end as agnostic as I could since there are so many Rust crates that let you display
pixels on the screen, and as a proof of concept, I experimented with two different graphical backends: `iced` (which
is the current `Maple //` GUI/graphics library) and `minifb`). Here is what the experiment looks like:

https://github.com/cbeust/sixty.rs/assets/92322/499b9f47-600c-4ce7-85b7-c373e18b427e

