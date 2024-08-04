use std::{fs};
use std::ops::{BitXor};
use crossbeam::channel::Sender;
use crate::disk::bit_stream::{Nibble};
use crate::constants::{HOLD, NibbleStrategy::*, NIBBLE_STRATEGY};
use crate::cycle_actions::{Actions, UpdatePhaseAction};
use crate::cycle_actions::CycleAction::{UpdatePhase};
use crate::disk::disk::{Disk};
use crate::{send_message};
use crate::disk::disk_info::DiskInfo;
use crate::disk::drive::{Drive};
use crate::messages::ToUi;
use crate::ui::iced::shared::*;
use crate::messages::ToUi::{DiskSelected, FirstRead};

/// Divide by 2 to get the phase, by 4 to get the track
pub const MAX_PHASE: usize = 160;
/// 40 tracks max
pub const MAX_TRACK: usize = MAX_PHASE / 4;
/// A .dsk only has 35 tracks, not 40
pub const MAX_TRACK_DSK: usize = MAX_TRACK - 5;

/// Size of a sector
pub const SECTOR_SIZE_BYTES: usize = 256;
/// A track has 16 sectors of 256 bytes each
pub const TRACK_SIZE_BYTES: usize = 16 * SECTOR_SIZE_BYTES;
/// Size of a disk
pub const DSK_SIZE_BYTES: usize = TRACK_SIZE_BYTES * MAX_TRACK_DSK;
/// Size of the data field (following D5 AA AD)
pub const DATA_FIELD_SIZE: usize = 343;

/* DO logical order  0 1 2 3 4 5 6 7 8 9 A B C D E F */
/*    physical order 0 D B 9 7 5 3 1 E C A 8 6 4 2 F */
/* PO logical order  0 E D C B A 9 8 7 6 5 4 3 2 1 F */
/*    physical order 0 2 4 6 8 A C E 1 3 5 7 9 B D F */
// let LOGICAL_SECTORS: Vec<u8> = vec![0, 0xD, 0xB, 9, 7, 5, 3, 1, 0xE, 0xC, 0xA, 8, 6, 4, 2, 0xF];
pub const LOGICAL_SECTORS: [u8; 16] = [0, 7, 14, 6, 13, 5, 12, 4, 11, 3, 10, 2, 9, 1, 8, 15];
pub const LOGICAL_SECTORS_WRITE: [u8; 16]
    = [0, 0xd, 0xb, 9, 7, 5, 3, 1, 0xe, 0xc, 0xa, 8, 6, 4, 2, 0xf];

#[derive(Default)]
pub(crate) struct DiskController {
    /// 1-6
    slot: u8,

    drives: [Drive; 2],
    drive_index: usize,

    // phase: u8,
    magnet_states: u8,
    // Current track * 2 (range: 0-79)
    drive_phase_80: usize,

    sender: Option<Sender<ToUi>>,

    /// Keep track of the sector we are currently on
    sector_read: SectorRead,

    /// Next byte to return
    latch: u8,
    pub(crate) missed_bytes: u32,

    clock: u128,
    /// Keep track of the last time we wrote, so we can insert sync bits if necessary
    previous_write_clock: u128,

    /// How long to hold to a byte
    hold: u16,

    /// LSS
    lss: crate::disk::lss::Lss,
    q6: bool,
    q7: bool,
    next_qa: u8,

    /// Used by the next bit algorithm
    head_window: u8,

    /// Deferred actions (e.g. moving the head, spinning down the drive)
    actions: Actions,

    /// If true, we wrote something to disk so we need to save the new content
    /// to the file as soon as we're switching back to read mode
    write_dirty: bool,
    /// Next byte to be written
    write_load: u8,

    /// Keep track of the first time we read a phase. Indexed by the disk drive
    first_time_reading_phase: [u8; 2],

}

impl DiskController {
    pub(crate) fn new_with_filename(slot: u8, disk_infos: &[Option<DiskInfo>; 2],
            sender: Option<Sender<ToUi>>) -> Self {
        let mut result = Self {
            slot,
            sender,
            ..Default::default()
        };
        if let Some(di) = Shared::get_drive(0) {
            result.load_disk_from_file(0, di);
        }
        if let Some(di) = Shared::get_drive(1) {
            result.load_disk_from_file(1, di);
        }
        result

    }

    pub fn load_disk_new(path: &str, sender: Option<Sender<ToUi>>) -> Result<Disk, String> {
        Disk::new(path, true /* don't read bit_streams */, sender)
    }

    pub(crate) fn load_disk_from_file(&mut self, drive_number: usize, disk_info: DiskInfo)
    {
        if DiskController::file_to_bytes(drive_number, &disk_info, &self.sender).is_some() {
            match Disk::new(&disk_info.path, false /* read bit_streams */, self.sender.clone()) {
                Ok(disk) => {
                    self.drives[drive_number].disk = Some(disk);
                    Self::on_new_disk_info(self.sender.clone(), drive_number, Some(disk_info));
                }
                Err(error) => {
                    println!("Couldn't load disk: {}", error);
                    Self::on_new_disk_info(self.sender.clone(), drive_number, None);
                }
            }
        }
    }

    pub(crate) fn swap_disks(&mut self) {
        let disk_info_0 = self.drives[0].disk.clone().map(|d| d.disk_info());
        let disk_info_1 = self.drives[1].disk.clone().map(|d| d.disk_info());
        println!("Swapping drive {:?} and {:?}", &disk_info_0, &disk_info_1);
        let tmp = self.drives[0].clone();
        self.drives[0] = self.drives[1].clone();
        self.drives[0].drive_number = 0;
        Self::on_new_disk_info(self.sender.clone(), 0, disk_info_1);
        self.drives[1] = tmp;
        self.drives[1].drive_number = 1;
        Self::on_new_disk_info(self.sender.clone(), 1, disk_info_0);
    }

    pub fn disks(&self) -> [Option<DiskInfo>; 2] {
        self.drives.clone().map(|drive| drive.disk.map(|disk| disk.disk_info()))
    }

    fn on_new_disk_info(sender: Option<Sender<ToUi>>, drive_number: usize,
            disk_info: Option<DiskInfo>)
    {
        Shared::set_drive(drive_number, disk_info.clone());
        send_message!(&sender, ToUi::DiskInserted(drive_number, disk_info));
    }

    pub fn file_to_bytes(drive_number: usize, disk_info: &DiskInfo, sender: &Option<Sender<ToUi>>)
            -> Option<Vec<u8>> {
        match fs::read(disk_info.path()) {
            Ok(f) => {
                println!("Loading \"{}\", {}, in drive {}", disk_info.name(), disk_info.path(),
                    drive_number);
                Self::on_new_disk_info(sender.clone(), drive_number, Some(disk_info.clone()));
                Some(f)
            }
            Err(_) => {
                println!("Couldn't load \"{}\", {}, in drive {}", disk_info.name(),
                    disk_info.path(), drive_number);
                Self::on_new_disk_info(sender.clone(), drive_number, None);
                None
            }
        }
    }

    fn log(&self, s: &str) {
        // println!("[DiskController {:>4}] {}", self.clock, s);
    }

    fn is_motor_on(&self) -> bool {
        self.drives[self.drive_index].is_on()
    }

    /// Go through all the deferred actions in the list and either decrease their
    /// waiting time, or execute them if that counter has reached zero.
    /// Once this is done, remove the actions that have run.
    fn execute_actions(&mut self) {
        let mut new_bit_position: Option<usize> = None;
        let current_bit_position = self.current_bit_position();

        for wrapper in &mut self.actions.actions {
            use crate::cycle_actions::CycleAction::*;
            if wrapper.wait == 0 {
                match &wrapper.action {
                    UpdatePhase(v) => {
                        #[cfg(feature = "log_disk")]
                        if self.drives[v.drive_index].get_phase_160() != v.phase_160 {
                            log::info!("@@ updatePhase={}->{}",
                                self.drives[v.drive_index].get_phase_160(), v.phase_160);
                        }

                        self.drives[v.drive_index].set_phase_160(v.phase_160);
                        if let Some(disk) = &self.drives[v.drive_index].disk {
                            let old_phase = self.drives[v.drive_index].get_phase_160();
                            let new_phase =  v.phase_160;
                            let old_length = & disk.get_stream_len(old_phase);
                            let new_length = & disk.get_stream_len(new_phase);
                            new_bit_position =
                                Some((current_bit_position * new_length / old_length) % new_length);
                            // println!("Old position: {}  new: {:#?}", current_bit_position, new_bit_position);
                        }
                        Shared::set_phase_160(v.drive_index, v.phase_160 as u8);
                    }
                    MotorOff(v) => {
                        if ! wrapper.has_run {
                            self.drives[v.drive_index].turn_off();
                        }
                    }
                }
                wrapper.has_run = true;
            } else {
                wrapper.wait -= 1;
            }
        }

        if let Some(nbp) = new_bit_position {
            self.set_current_bit_position(nbp);
        }

        self.actions.actions.retain(|a| ! a.has_run);
    }

    pub(crate) fn step(&mut self) {
        // Only every other clock cycle since we're going twice as fast as the CPU
        if (self.clock % 2) == 0 {
            self.execute_actions();
        }

            match NIBBLE_STRATEGY {
            Lss => {
                /// TODO: should take into account whether the motor is on and not pass true
                /// but it needs to be delayed or Sherwood Forest won't boot
                self.lss.on_pulse(self.q6, self.q7, true, &mut self.drives[self.drive_index]);
                self.latch = self.lss.latch;
                self.clock = self.clock.wrapping_add(1);
            }
            Bits => {
                if self.is_motor_on() {
                    if (self.clock % 8) == 0 {
                        let new_bit = self.next_bit_with_window();
                        if (self.latch & 0x80) > 0 { // qa is set
                            if new_bit == 0 && self.next_qa == 0 {
                                // do nothing, this is how we sync
                            } else if new_bit == 1 && self.next_qa == 0 {
                                self.next_qa = 1;
                            } else if self.next_qa == 1 {
                                self.latch = 2 | new_bit;
                                self.next_qa = 0;
                            }
                        } else {
                            // qa not set
                            self.latch = (self.latch << 1) | new_bit;
                            // println!("Clock: {}, shifting bit: {} latch: {:02X}", self.clock, new_bit, self.latch)
                        }
                    }
                    self.clock = self.clock.wrapping_add(1);
                    #[cfg(feature = "log_disk")]
                    log::info!("@@ clock={} bitPosition={} latch={:02X}", self.clock,
                        self.current_bit_position(), self.latch);
                }
            }
            Bytes => {
                if self.is_motor_on() {
                    if self.hold > 0 {
                        // println!("Holding byte {:02X} for {} more", self.latch, self.hold);
                        self.hold -= 1;
                    } else {
                        // println!("Missed byte {:02X}", self.latch);
                        // missed byte
                        self.latch = self.next_byte();
                        self.hold = HOLD;
                    }
                }
            }
        }
    }

    fn set_current_bit_position(&mut self, position: usize) {
        if let Some(ref mut disk) = &mut self.drives[self.drive_index].disk {
            disk.bit_position = position;
        }
    }

    fn current_bit_position(&self) -> usize {
        if let Some(disk) = &self.drives[self.drive_index].disk {
            disk.bit_position
        } else {
            0
        }
    }

    pub(crate) fn accept(&self, address: u16) -> bool {
        if address >= 0xc080 {
            let a = address - (self.slot * 16) as u16;
            a >= 0xc080 && a <= 0xc08f
        } else {
            false
        }
    }

    pub(crate) fn get_or_set(&mut self, get: bool, address: u16, value: u8,
            sender: &Option<Sender<ToUi>>)
            -> u8 {
        let address = address - (self.slot * 16) as u16;
        match address {
            0xc080..=0xc087 => {
                if self.is_motor_on() {
                    // Stepper motor
                    // println!("Stepper motor: {:04X}", address);
                    self.update_stepper(address);
                }
                self.latch
            }
            0xc088 => {
                // log_emulator(&format!("Turn motor off: {:02X}", self.latch));
                // log::info!("Turning {} drive off, latch: {:02X}",
                //     if self.disk_index == 0 { "left" } else { "right" }, self.latch);

                // Turn motor off.
                if self.drives[self.drive_index].turn_off() {
                    // If turn_off() returned true, we are just transitioning from On -> SpinningDown
                    // Schedule an action later to transition from SpinningDown -> Off
                    // self.actions.add_action(SPINNING_DOWN_CYCLES, MotorOff(MotorOffAction {
                    //     drive_index: self.drive_index,
                    // }));
                }
                0
            }
            0xc089 => {
                // log_emulator(&format!("Turn motor on: {:02X}", self.latch));
                // log::info!("Turning {} drive on, latch: {:02X}",
                //     if self.disk_index == 0 { "left" } else { "right" }, self.latch);
                self.drives[self.drive_index].turn_on();
                self.actions.remove_motor_off_actions();
                0
            }
            0xc08a => {
                // log::info!("Switching to left drive");
                send_message!(&sender,DiskSelected(0));
                self.drive_index = 0;
                0
            }
            0xc08b => {
                // log::info!("Switching to right drive");
                send_message!(&sender, DiskSelected(1));
                self.drive_index = 1;
                // self.disks[self.disk_index].turn_on();
                0
            }
            0xc08c => {
                let drive_index = self.drive_index;
                let current_phase = Shared::get_phase_160(drive_index);
                if self.first_time_reading_phase[drive_index] != current_phase {
                    send_message!(&self.sender, FirstRead(drive_index, Shared::get_phase_160(drive_index)));
                    self.first_time_reading_phase[drive_index] = current_phase;
                }
                // Q6L
                self.q6 = false;
                if ! self.q7 {
                    // READ
                    let result = self.latch;

                    // Fill the latch
                    if (result & 0x80) > 0 {
                        self.sector_read.read_byte(drive_index, result, &self.sender);
                        if NIBBLE_STRATEGY != Bits {
                            self.latch = 0;
                        }
                    }
                    self.previous_write_clock = 0;
                    result
                } else {
                    // WRITE
                    if let Some(ref mut disk) = &mut self.drives[drive_index].disk {
                        // println!("Writing {:02X} at clock delta {}", self.write_load, self.clock - self.previous_write_clock);
                        let phase_160 = self.drive_phase_80 * 2;
                        let mut sync_bits: u16 = 0;
                        let delta = self.clock - self.previous_write_clock;
                        if self.previous_write_clock != 0 {
                            // if self.write_load == 0xff {
                            //     println!("FF sync");
                            // }
                            if delta != 64 && delta != 80 && delta != 72 {
                                println!("WEIRD SYNC BITS, DELTA: {}", delta);
                            }
                            let rest = (delta - 64) / 2;
                            if rest > 4 {
                                sync_bits = (rest / 4) as u16;
                        }
                        // } else if self.write_load == 0xff {
                        //     sync_bits = 2;
                        }
                        let nibble = Nibble::new(self.write_load, sync_bits);
                        // println!("Writing nibble {} at position {:04X}", nibble,
                        //     disk.bit_position / 8);
                        // if nibble.value != 0xff {
                        //     disk.bit_streams.dump(phase_160);
                        // }
                        // let mut stream = &mut disk.get_stream_mut(phase_160 as usize);
                        for bit in nibble.to_bits() {
                            disk.set_bit_and_advance(phase_160, bit);
                            // stream.set_bit(disk.bit_position, bit);
                            // println!("  Writing bit {} at position {:04X}", bit, disk.bit_position / 8);
                            // disk.bit_position = (disk.bit_position + 1) % stream.len();
                        }
                        // if nibble.value != 0xff {
                        //     disk.bit_streams.dump(phase_160);
                        // }

                        // Woz::save("d:\\Apple Disks\\save.woz", &disk.bit_streams);
                        // println!("Writing nibble {}", nibble);
                        // let value = self.write_load;
                        // for i in 0..8 {
                        //     let bit = bit(value, i);
                        //     stream.set_bit(disk.bit_position, bit);
                        //     disk.bit_position = (disk.bit_position + 1) % stream.len();
                        // }
                    }
                    self.previous_write_clock = self.clock;
                    self.write_dirty = true;
                    0
                }

            }
            0xc08d => {
                // Q6H
                self.q6 = true;
                self.lss.reset();
                if ! self.q7 {
                    0 /* 0x80 to indicate write protected */
                } else {
                    // write mode when Q7H
                    self.write_load = value;
                    // println!("Setting write_load to {value:02X}");
                    0
                }
            }
            0xc08e => {
                // Q7L
                self.q7= false;
                self.previous_write_clock = 0;
                let mut result = 0;
                if self.q6 {
                    if let Some(disk) = &self.drives[self.drive_index].disk {
                        if disk.disk_info().is_write_protected { result = 0xff; }
                    }
                }
                result
            }
            0xc08f => {
                // Q7H
                self.previous_write_clock = 0;
                self.q7 = true;
                self.write_load = value;
                0
            }
            _ => { panic!("Not a disk controller address"); }
        }
    }

    fn move_in_direction(current: u8, direction: i8) -> u8 {
        if direction == 0 {
            current
        } else if direction == 1 {
            if current < 79 { current + 1 }
            else { current }
        } else if direction == -1 {
            if current > 0 { current - 1 }
            else { current }
        } else {
            panic!("Should never happen");
        }
    }

    fn update_stepper(&mut self, address: u16) {
        // phase is 0..3
        let phase = (address >> 1) & 3;
        // phase_bit is 1000, 0100, 0010, 0001
        let phase_bit = 1 << phase;

        // update the magnet states
        if (address & 1) != 0 {
            // phase on
            self.magnet_states = self.magnet_states | phase_bit;
            #[cfg(feature = "log_disk")]
            log::info!("@@ magnetStates={}", self.magnet_states);
        } else {
            // phase off
            let inverse = phase_bit.bitxor(0xff);
            self.magnet_states = self.magnet_states & inverse;
            #[cfg(feature = "log_disk")]
            log::info!("@@ magnetStates={}", self.magnet_states);
        }
        self.magnet_states &= 0xf;

        // check for any stepping effect from a magnet
        // - move only when the magnet opposite the cog is off
        // - move in the direction of an adjacent magnet if one is on
        // - do not move if both adjacent magnets are on (ie. quarter track)
        // momentum and timing are not accounted for ... maybe one day!
        let mut direction: i8 = 0;
        if self.magnet_states & (1 << ((self.drive_phase_80 + 1) & 3)) != 0 {
            direction += 1;
        }
        if self.magnet_states & (1 << ((self.drive_phase_80 + 3) & 3)) != 0 {
            direction -= 1;
        }

        if direction != 0 {
            // If anything needs to be written, do that before moving the head
            if self.write_dirty {
                if let Some(ref mut disk) = self.drives[self.drive_index].disk {
                    disk.save();
                }
                self.write_dirty = false;
            }
        }

        // Set the track to drive_phase / 2
        // let new_track = min(79, self.drive_phase >> 1); // (round half tracks down)
        // if new_track != self.current_track {
        //     self.current_track = new_track;
        // }

        let mut quarter_direction = 0;
        if self.magnet_states == 0xc || self.magnet_states == 0x6 || self.magnet_states == 0x3 ||
                self.magnet_states == 9 {
            quarter_direction = direction;
            direction = 0;
        }

        // Update drive_phase too
        self.drive_phase_80 = DiskController::move_in_direction(self.drive_phase_80 as u8, direction) as usize;
        let mut new_phase_precise = self.drive_phase_80 as f32 + quarter_direction as f32 / 2.0;
        if new_phase_precise < 0.0 { new_phase_precise = 0.0; }
        // let current_phase = self.drives[self.drive_index].get_phase_160() / 2;

        // if false {
        //     // self.drives[self.drive_index].set_phase80(self.drive_phase_80);
        //     // Shared::set_phase_160(self.drive_index, (self.drive_phase_80 as u8) * 2);
        // } else {
        // if self.drive_phase_80 * 2 != (new_phase_precise * 2.0) as usize {
            // Move the head, but we need to insert a small delay for the first move if it wasn't
            // in motion already. For subsequent ones, the head can move right away (hence delay of 1
            // cycle)
            // Note: ArcticFox doesn't boot with this change, turn it off for now
            // let delay = if self.actions.actions.is_empty() && current_phase != self.drive_phase_80 { 30_000 } else { 1 };
            // if self.drive_phase_80 * 2 != (new_phase_precise * 2.0) as usize {
            //     println!("Phase drive: {} precise: {}", self.drive_phase_80 * 2, new_phase_precise * 2.0);
            // }
            self.actions.add_action(1, UpdatePhase(UpdatePhaseAction {
                drive_index: self.drive_index,
                phase_160: (new_phase_precise * 2.0) as usize, // self.drive_phase_80 * 2,
            }));
        // }

        #[cfg(feature = "log_disk")]
        log::info!("@@ direction={} phase={} magnetStates={}", direction, self.drive_phase_80, self.magnet_states);
    }

    fn track(&self) -> usize {
        self.drive_phase_80 >> 1
    }

    // pub(crate) fn set_bit_position(&mut self, bit_position: usize) {
    //     self.bit_index = bit_position;
    // }

    fn next_bit_with_window(&mut self) -> u8 {
        let bit = self.next_bit();
        self.head_window = (self.head_window << 1) | bit;
        if (self.head_window & 0x0f != 0) {
            (self.head_window & 0x02) >> 1
        } else {
            use rand::prelude::*;
            if random::<f32>() < 0.3 { 1 } else { 0 }
        }
    }

    fn next_bit(&mut self) -> u8 {
        // log::info!("Getting next bit from disk {}", self.disk_index);
        self.drives[self.drive_index].disk.clone()
            .map_or(0, |mut d| d.next_bit(self.drives[self.drive_index].get_phase_160()))
    }

    // fn current_bit_index(&self) -> usize {
    //     self.bit_index
    // }

    pub(crate) fn next_byte(&mut self) -> u8 {
        let mut result = 0;
        let mut count = 0;
        // while count < 10 && (result & 0x80) == 0 {
        for _ in 0..8 {
        // while (result & 0x80) == 0 {
            result = (result << 1) | self.next_bit();
            count += 1;
        }
        // if result == 0xd5 || result == 0xaa {
        //     println!("Read {:02x} at {:?}", result, self.bit_buffers[self.track() as usize]);
        // }

        // println!("Read {:02x} at {:?}", result, self.bit_buffers[self.track() as usize]);
        self.sector_read.read_byte(self.drive_index, result, &self.sender);
        result
    }

    pub fn left_disk(&self) -> &Option<Disk> {
        &self.drives[0].disk
    }

}

#[derive(PartialEq, Debug, Clone, Copy)]
enum State {
    START, D5, D5AA, VOLUME0, VOLUME1, TRACK0, TRACK1, SECTOR0, SECTOR1, CHECKSUM0,
}

struct SectorRead {
    state: State,
    current_byte: u8,
    volume: u8,
    track: u8,
    sector: u8,
}

impl Default for SectorRead {
    fn default() -> Self {
        Self {
            state: State::START,
            current_byte: 0,
            volume: 0,
            track: 0,
            sector: 0,
        }
    }
}

impl SectorRead {
    fn read_byte(&mut self, disk_index: usize, byte: u8, sender: &Option<Sender<ToUi>>) {
        use State::*;

        fn pair(b0: u8, b1: u8) -> u8 {
            ((b0 << 1) | 1) & b1
        }

        match byte {
            0xd5 if self.state == START => { self.state = D5; }
            0xaa if self.state == D5 => { self.state = D5AA; }
            0x96 if self.state == D5AA => { self.state = VOLUME0; }
            _ => {
                match self.state {
                    VOLUME0 => {
                        self.current_byte = byte;
                        self.state = VOLUME1;
                    }
                    VOLUME1 => {
                        self.volume = pair(self.current_byte, byte);
                        self.state = TRACK0;
                    }
                    TRACK0 => {
                        self.current_byte = byte;
                        self.state = TRACK1;
                    }
                    TRACK1 => {
                        self.track = pair(self.current_byte, byte);
                        self.state = SECTOR0;
                    }
                    SECTOR0 => {
                        self.current_byte = byte;
                        self.state = SECTOR1;
                    }
                    SECTOR1 => {
                        self.sector = pair(self.current_byte, byte);
                        self.state = START;
                        Shared::set_sector(disk_index, self.sector);
                        Shared::set_track(disk_index, self.track);
                        // println!();
                    }
                    _ => {
                        self.state = START;
                    }
                }
            }
        }
        // println!("read_byte: {:02X} state: {:?} -> {:?}", byte, old_state, self.state);
    }
}
