use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;

use cpal::{Device, FromSample, Sample, Stream, StreamConfig, traits::{DeviceTrait, HostTrait, StreamTrait}};
use itertools::Itertools;
use rodio::{dynamic_mixer, OutputStream, Sink, Source};
use crate::alog::alog;

use crate::ui::iced::shared::Shared;

pub struct Speaker2 {
    sink: Sink,
    /// The last time we detected an access to C03X
    last_cycle: u64,
}

// impl Speaker2 {
//     pub fn new() -> Self {
//         let (controller, mixer) = dynamic_mixer::mixer::<f32>(1, 44_100);
//         let (_stream, stream_handle) = OutputStream::try_default().unwrap();
//         let sink = Sink::try_new(&stream_handle).unwrap();
//         let source = AStream{};
//         sink.append(source);
//         Speaker2 { sink, last_cycle: 0, speaker_on: false, }
//     }
// 
//     pub fn run(&self) {
//         // controller.add(source);
//         self.sink.set_volume(2.0);
// 
//         // Sleep the thread until sink is empty.
//         self.sink.play();
//         self.sink.sleep_until_end();
// 
//         // loop {
//         //     std::thread::sleep(std::time::Duration::from_millis(100));
//         // }
//         println!("Exiting Speaker2");
//     }
// 
//     pub fn play() {
//         let (_stream, stream_handle) = OutputStream::try_default().unwrap();
//         let sink = Sink::try_new(&stream_handle).unwrap();
//         let source = AStream{};
//         sink.append(source);
//         sink.sleep_until_end();
//     }
// 
//     /// cycles is guaranteed to not be empty
//     pub fn cycles_to_samples(&mut self, cycles: Vec<u64>, sample_frequency: u64) -> Vec<f32> {
//         let mut result: Vec<f32> = Vec::new();
//         // let duration_ms: f32 = (cycles[cycles.len() - 1] as f32 - cycles[0] as f32) * 1000.0
//         //     / cpu_frequency as f32;
// 
//         // The value used to downsample. The cycles are recorded at CPU speed (1Mhz)
//         // but the sample rate is around 44kHz, so this number is 1M / 44k ~= 23.
//         // 23 CPU samples turn into one sound sample
//         // let sample_frequency = self.config.sample_rate.0;
//         let cpu_frequency = 1_000_000;
//         let sampling = cpu_frequency / sample_frequency as u64;
//         if (cycles[0] - self.last_cycle) > 100_000 {
//             println!("Long silence");
//         }
//         if self.last_cycle == 0 || (cycles[0] - self.last_cycle) > 100_000 {
//             self.last_cycle = cycles[0];
//         }
//         let mut intervals: Vec<u64> = Vec::new();
//         intervals.push(cycles[0] - self.last_cycle);
//         for i in 1..cycles.len() {
//             intervals.push(cycles[i] - cycles[i - 1]);
//         }
//         self.last_cycle = cycles[cycles.len() - 1];
//         for i in intervals {
//             let sound_sample_count = i as f32 / sampling as f32;
//             let value = if self.speaker_on {
//                 0.2
//             } else {
//                 -0.2
//             };
//             // alog(&format!("Interval: {i}, adding #{} of sample {}", sound_sample_count.round(), value));
//             for j in 0..sound_sample_count as usize {
//                 result.push(value);
//             }
//             self.speaker_on = ! self.speaker_on;
//         }
//         // for cycle in cycles {
//         //         let total_cycles = cycle - self.last_cycle;
//         //         self.last_cycle = cycle;
//         //         let sound_sample_count = total_cycles as f32 / sampling as f32;
//         //             // if sound_sample_count < 1.0 {
//         //             //     println!("Low count");
//         //             // }
//         //         let mut just_transitioned = false;
//         //         for _ in 0..sound_sample_count.round() as usize {
//         //             // let value = if just_transitioned {
//         //             //     just_transitioned = false;
//         //             //     sampling as f32 / 2.0
//         //             // } else
//         //             let value = if self.speaker_on {
//         //                 1.0
//         //             } else {
//         //                 -1.0
//         //             };
//         //             Shared::add_sound_sample(value);
//         //         }
//         //         // alog(&format!("Speaker: count:{sound_sample_count} sample:{}, duration:{duration_ms}",
//         //         //     if self.speaker_on { sampling } else { 0 }));
//         //         self.speaker_on = ! self.speaker_on;
//         //         just_transitioned = true;
//         // }
// 
//         result
//     }
// 
// }

#[derive(Default)]
pub struct Samples {
    last_cycle: u64,
    speaker_on: bool,
}

impl Samples {
    /// cycles is guaranteed to not be empty
    pub fn cycles_to_samples(&mut self, cycles: Vec<u64>, sample_frequency: u64) -> Vec<f32> {
        let max = 0.1;
        let min = -0.1;
        let mut result: Vec<f32> = Vec::new();
        let cpu_frequency = 1_000_000;
        let sampling = cpu_frequency / sample_frequency;
        if self.last_cycle == 0 || (cycles[0] - self.last_cycle) > 1_000_000 {
            self.last_cycle = cycles[0];
        }
        let mut intervals: Vec<u64> = Vec::new();
        intervals.push(cycles[0] - self.last_cycle);
        for i in 1..cycles.len() {
            intervals.push(cycles[i] - cycles[i - 1]);
        }
        self.last_cycle = cycles[cycles.len() - 1];
        for i in intervals {
            let value = if self.speaker_on {
                max
            } else {
                min
            };
            let mut sound_sample_count = i as f32 / sampling as f32;
            if sound_sample_count < 1.0 { sound_sample_count = 1.0; }
            // alog(&format!("Interval: {i}, adding #{} of sample {}", sound_sample_count.round(), value));
            for _ in 0..sound_sample_count.round() as usize {
                result.push(value as f32);
            }
            // Smooth the transition between the two extreme values
            if ! result.is_empty() {
                self.speaker_on = !self.speaker_on;
            }
        }

        result
    }

}

pub struct Speaker {
    // speaker_on: bool,
    // device: Device,
    // config: SupportedStreamConfig,
    // channels: usize,
    pub config: StreamConfig,
    pub device: Device,
    pub channels: usize,
    last_sample: f32,

    /// The last time we detected an access to C03X
    stream: Stream,
}

impl Speaker {
    pub fn run() {
        let speaker = Speaker::new();
        speaker.stream.play().unwrap();

        loop {
            thread::sleep(Duration::from_millis(100));
        }
    }

    fn new() -> Self {
        let host = cpal::default_host();
        let device = host.default_output_device().unwrap();
        let config = device.default_output_config().unwrap();
        let channels = config.channels() as usize;

        // println!("Default output config: {:?}, sample rate: {sample_rate}", config);

        // let stream = device.build_output_stream(&config.into(), write_silence, err_fn, None).unwrap();
        let config: StreamConfig = config.clone().into();

        let mut next_value_2 = move || -> f32 {
            Shared::get_next_sound_sample()
        };

        let err_fn = |err| eprintln!("an error occurred on stream: {}", err);
        let stream = device.build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                Self::write_data(data, channels, &mut next_value_2)
            },
            err_fn,
            None,
        ).unwrap();

        Self {
            last_sample: 0.0,
            device, config, channels, stream,

        }
    }

    pub fn play_file(&mut self, path: &str) {
        let samples = file_to_samples(path);
        for s in samples {
            Shared::add_sound_sample(s);
        }
        println!("Playing file {path}");
        self.stream.play().unwrap();
        thread::sleep(Duration::from_millis(5_000));
    }

    fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> f32)
        where
            T: Sample + FromSample<f32>,
    {
        for frame in output.chunks_mut(channels) {
            let s = next_sample();
            let value: T = T::from_sample(s);
            for sample in frame.iter_mut() {
                *sample = value;
            }
        }
    }
}

pub fn file_to_samples(path: &str) -> Vec<f32> {
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);
    let mut cycles: Vec<u64> = Vec::new();
    for line in reader.lines() {
        let l = line.unwrap_or_default();
        if let Ok(n) = str::parse::<u64>(&l) {
            if n != 0 {
                cycles.push(n);
            }
        }
    }

    // println!("Samples: {} {}", samples[0], samples[1]);

    let speaker = Speaker::new();
    Samples::default().cycles_to_samples(cycles, speaker.config.sample_rate.0.into())
}


pub fn play_file_rodio(path: &str) {
    let samples = file_to_samples(path);
    for s in samples {
        Shared::add_sound_sample(s);
    }

    // let (controller, mixer) = dynamic_mixer::mixer::<f32>(1, 48_100);
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    let source = AStream{};
    // controller.add(source);

    // Append the dynamic mixer to the sink to play a C major 6th chord.
    sink.append(source); // mixer);

    // Sleep the thread until sink is empty.
    sink.sleep_until_end();

}

struct AStream {}
impl Source for AStream {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        48_100
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

impl Iterator for AStream {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        // println!(" RETURNING NEXT");
        Shared::get_next_sound_sample_maybe()
    }
}
