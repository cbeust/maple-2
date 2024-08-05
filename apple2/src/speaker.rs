use cpal::{traits::{DeviceTrait, HostTrait, StreamTrait}, FromSample, Sample, SizedSample, Device, BuildStreamError, Stream};

pub fn init_speaker() {
    let host = cpal::default_host();

    let device = host.default_output_device().unwrap();
    // println!("Output device: {}", device.name()?);

    let config = device.default_output_config().unwrap();
    println!("Default output config: {:?}", config);

    let sample_rate = config.sample_rate().0 as f32;
    let channels = config.channels() as usize;
    //
    // // Produce a sinusoid of maximum amplitude.
    let mut sample_clock = 0f32;
    let mut sample = 0.5;
    let mut next_value = move || {
        sample_clock = (sample_clock + 1.0) % sample_rate;
        let mut result = (sample_clock * 440.0 * 2.0 * std::f32::consts::PI / sample_rate).cos();
        // sample = -sample;
        result = sample;

        println!("Next value: {result} clock:{sample_clock}");
        result
    };

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let stream = device.build_output_stream(
        &config.into(),
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            write_data(data, channels, &mut next_value)
        },
        err_fn,
        None,
    ).unwrap();
    stream.play().unwrap();

    std::thread::sleep(std::time::Duration::from_millis(1000));
}

fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> f32)
    where
        T: Sample + FromSample<f32>,
{
    for frame in output.chunks_mut(channels) {
        let value: T = T::from_sample(next_sample());
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}
