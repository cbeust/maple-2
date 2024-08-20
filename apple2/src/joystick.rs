use std::sync::{Arc, RwLock};
use std::time::Duration;

use gilrs::{Axis, Button, Gilrs};
use gilrs::ev::Code;
use gilrs::EventType::{AxisChanged, ButtonChanged, ButtonPressed, ButtonReleased};
use crate::send_message;
use crate::ui::iced::shared::Shared;

pub struct Joystick {
    // _gilrs: Arc<RwLock<Gilrs>>,
    reset: bool,
    // Reset cycles for all four paddles (cycles)
    reset_cycles: [u64; 4],
    // Controller values are 0-255
    // controller_values: [u8; 4],
}

impl Default for Joystick {
    fn default() -> Self {
        Self {
            // _gilrs: Arc::new(RwLock::new(Gilrs::new().unwrap())),
            reset: false,
            reset_cycles: [0, 0, 0, 0],
            // controller_values: [128, 128, 128, 128],
            // y_cycle: Some(MULT * 128),
        }
    }
}
const MULT: u64 = 12;

impl Joystick {
    pub fn reset_cycles(&mut self, cycle: u64) {
        self.reset_cycles.fill(cycle);
    }

    pub fn run(&mut self) {
        loop {
            self.main_loop();
        }
    }

    /// Run a forever loop that constantly updates the raw values from the controller
    pub fn main_loop(&mut self) {
        // Map the -1..1 range to 0..255
        fn scale(x: f32, invert: bool) -> u8 {
            let result = ((128.0 * x) + 128.0) as u8;
            if invert { 255 - result } else { result }
        }

        let mut gilrs = Gilrs::new().unwrap();
        while let Some(gilrs::Event { id, event, time }) = gilrs.next_event_blocking(
            Some(Duration::from_millis(100)))
        {
            let mut values = Shared::get_controller_raw_values();
            match event {
                AxisChanged(axis, value, _code) => {
                    match axis {
                        Axis::LeftStickX => {
                            values[0] = scale(value, false);
                        }
                        Axis::LeftStickY => {
                            values[1] = scale(value, true);
                        }
                        _ => {
                            // println!("Unknown event: {event:#?}");
                        }
                    }
                }
                ButtonPressed(button, _code) | ButtonReleased(button, _code) => {
                    let pressed = matches!(event, ButtonPressed(_, _));
                    match button {
                        Button::South => { Shared::set_controller_button_value(0, pressed); }
                        Button::West => { Shared::set_controller_button_value(1, pressed); }
                        _ => {}
                    };
                }
                _ => {}
            }
            Shared::update_controller_raw_values(values);
        }

        //
        // if let Some(gamepad) = active_gamepad.map(|id| gilrs.gamepad(id)) {
        //     let mut result: Vec<u8> = Vec::new();
        //     let state = gamepad.state();
        //     for (index, axis) in [Axis::LeftStickX, Axis::LeftStickY, Axis::RightStickX, Axis::RightStickY].iter().enumerate() {
        //         let live_axis = gamepad.axis_data(*axis);
        //         let cached_axis = state.axis_data(gamepad.axis_code(*axis).unwrap());
        //         let float_value = match (live_axis, cached_axis) {
        //             (Some(live), Some(cached)) => {
        //                 println!("Live: {} cached: {}", live.value(), cached.value());
        //                 live.value()
        //             }
        //             (Some(live), None) => {
        //                 println!("Live:: {}", live.value());
        //                 live.value()
        //             }
        //             (None, Some(cached)) => {
        //                 println!("Cached: {}", cached.value());
        //                 cached.value()
        //             }
        //             (None, None) => { 0.0 }
        //         };
        //         result.push(scale(float_value, false));
        //     }
        //     println!("Updating to values {} {}", result[0], result[1]);
        //     Shared::update_controller_raw_values([result[0], result[1], result[2], result[3]]);
        // };

    }
//
    //     pub fn reset_timers(&mut self, cycle: u64) {
//         self.reset_cycles.fill(cycle);
//         for i in 0..4 {
//             self.controller_values[i] = Some(self.read_controller(i));
//         }
//     }
//

    pub fn get_value_for_paddle(&mut self, paddle_index: usize, current_cycle: u64) -> u8 {
        let values = Shared::get_controller_raw_values();
        let result = if self.reset_cycles[paddle_index] > 0 {
            if current_cycle > self.reset_cycles[paddle_index] + 256 * MULT {
                self.reset_cycles[paddle_index] = 0;
                0
            } else {
                let v = values[paddle_index];
                if current_cycle > self.reset_cycles[paddle_index] + v as u64 * MULT {
                    self.reset_cycles[paddle_index] = 0;
                    0
                } else {
                    // println!("Returning 0x80");
                    0x80
                }
            }
        } else {
            0
        };

        result
    }

}