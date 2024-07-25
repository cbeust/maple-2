use std::fmt::{Display, Formatter};
use crossbeam::channel::Sender;
use crate::disk::disk::{Disk};
use crate::messages::ToUi;
use crate::messages::ToUi::DriveMotorStatus;
use crate::send_message;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum DriveStatus {
    On,
    #[default]
    Off,
    SpinningDown
}

impl Display for DriveStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            DriveStatus::On =>  { "On           " }
            DriveStatus::Off => { "Off          " }
            DriveStatus::SpinningDown =>    { "Spinning down" }
        };
        f.write_str(&s).unwrap();
        Ok(())
    }
}

#[derive(Default, Clone)]
pub struct Drive {
    /// 0 or 1
    pub drive_number: usize,

    pub disk: Option<Disk>,

    _status: DriveStatus,

    phase160: usize,

    // phase: u8,
    magnet_states: u8,
    // Current track * 2 (range: 0-79)
    drive_phase: u8,
    current_phase: u8,

    sender: Option<Sender<ToUi>>,
}

impl Drive {
    pub fn new(drive_number: usize, disk: Option<Disk>, sender: Option<Sender<ToUi>>) -> Self {
        Self {
            drive_number, disk, sender, ..Default::default()
        }
    }

    pub fn is_on(&self) -> bool {
        self._status != DriveStatus::Off
    }

    fn set_status(&mut self, status: DriveStatus) {
        self._status = status;
    }

    pub fn set_phase_160(&mut self, phase_160: usize) {
        self.phase160 = phase_160
    }

    pub fn get_phase_160(&self) -> usize {
        self.phase160
    }

    pub fn turn_on(&mut self) {
        self.set_status(DriveStatus::On);
        send_message!(&self.sender, DriveMotorStatus(self.drive_number, DriveStatus::On));
    }

    /// Return true if we need to delay
    pub fn turn_off(&mut self) -> bool {
        use DriveStatus::*;
        if self._status == On {
            self.set_status(SpinningDown);
            send_message!(&self.sender, DriveMotorStatus(self.drive_number, SpinningDown));
            true
        } else if self._status == SpinningDown {
            // We don't want to turn off if the status is On (which means
            // we might have been spinning down and then the motor was turned
            // back on. In such a case, do nothing)
            self.set_status(Off);
            send_message!(&self.sender, DriveMotorStatus(self.drive_number, Off));
            false
        } else {
            false
        }
    }
}
