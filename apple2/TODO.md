# TODO

- [ ] Replace stack display in Disassembly Line with JSR addresses
- [ ] Improve the debugger window
    - [ ] Navigate assembly listing
    - [ ] Edit memory
    - [ ] Breakpoints
    - [ ] Watchpoints
- [ ] Improve the Memory window
    - [ ] Show ASCII characters
    - [ ] Implement navigation ("go to address")
    - [ ] Allow editing
- [ ] Fix the `Dev` window suddenly expanding
- [ ] Timings for UpdatePhase messages
- [ ] GR
- [ ] Sound
- [ ] Experiment with swapping the various memory banks when the flip is switched, instead of waiting until access
(`mem::swap()`)
- [ ] Disk write
- [ ] LSS: the motor is always on, need to pass a delayed "is_motor_on()" so that Sherwood Forest will boot
- [ ] Load directly from `Woz a day` website
- [ ] Make sure the emulator can boot for everyone
- [ ] Switch to `minifb` fully for better keyboard control
    - [ ] Verify # and other shifted symbols
    - [ ] Verify control key
- [ ] Config file, and put it as a file watcher
- [ ] The config file should include directories where to look for disks, and display that in the main UI
- [ ] Replace all `0x` addresses in `memory.rs` with their symbol to avoid mistakes
- [ ] Implement joysticks (or at least emulation)
- [ ] Fix `FLASH`
- [ ] Make sure we can boot all the protected WOZ files listed in "`woz_test_images`"
- [ ] Support for disk formats:
    - [ ] .po
    - [ ] .nib
- [ ] Joysticks / paddles
- [ ] Write tests that try to boot disks and fail at cycle count and succeed at PC

# Done

- [x] Nibbles window
- [x] HGR improvements
- [x] 80 columns
