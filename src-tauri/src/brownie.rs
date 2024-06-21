use std::{
    sync::{
        atomic::{self, AtomicBool},
        mpsc,
    },
    thread,
};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    FromSample, Sample, SizedSample, Stream,
};
use rand::prelude::*;

enum Command {
    Play,
    Pause,
}

pub struct Brownie {
    sender: mpsc::Sender<Command>,
    playing: AtomicBool,
}

impl Brownie {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel::<Command>();
        // Spawn a new thread to listen for commands
        thread::spawn(move || {
            let stream = create_stream().unwrap();
            while let Ok(command) = receiver.recv() {
                match command {
                    Command::Play => {
                        stream.play().unwrap();
                    }
                    Command::Pause => {
                        stream.pause().unwrap();
                    }
                }
            }
        });

        Brownie {
            sender,
            playing: AtomicBool::new(false),
        }
    }

    pub fn play(&self) {
        if self.playing.load(atomic::Ordering::Relaxed) == false {
            self.playing.store(true, atomic::Ordering::Relaxed);
            self.sender.send(Command::Play).unwrap();
        }
    }

    pub fn pause(&self) {
        if self.playing.load(atomic::Ordering::Relaxed) == true {
            self.playing.store(false, atomic::Ordering::Relaxed);
            self.sender.send(Command::Pause).unwrap();
        }
    }

    pub fn is_playing(&self) -> bool {
        self.playing.load(atomic::Ordering::Relaxed)
    }
}

fn create_stream() -> anyhow::Result<Stream> {
    #[cfg(any(not(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd"
    )),))]
    let host = cpal::default_host();

    let device = host
        .default_output_device()
        .expect("failed to find output device");

    let config = device.default_output_config().unwrap();
    match config.sample_format() {
        cpal::SampleFormat::I8 => run::<i8>(&device, &config.into()),
        cpal::SampleFormat::I16 => run::<i16>(&device, &config.into()),
        cpal::SampleFormat::I32 => run::<i32>(&device, &config.into()),
        cpal::SampleFormat::I64 => run::<i64>(&device, &config.into()),
        cpal::SampleFormat::U8 => run::<u8>(&device, &config.into()),
        cpal::SampleFormat::U16 => run::<u16>(&device, &config.into()),
        cpal::SampleFormat::U32 => run::<u32>(&device, &config.into()),
        cpal::SampleFormat::U64 => run::<u64>(&device, &config.into()),
        cpal::SampleFormat::F32 => run::<f32>(&device, &config.into()),
        cpal::SampleFormat::F64 => run::<f64>(&device, &config.into()),
        sample_format => panic!("Unsupported sample format '{sample_format}'"),
    }
}

fn run<T>(device: &cpal::Device, config: &cpal::StreamConfig) -> Result<Stream, anyhow::Error>
where
    T: SizedSample + FromSample<f32>,
{
    let sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;
    let mut current_value = 0.0;
    let mut t = 0.0;
    let mut lpf = LowPassFilter::new(50.0, sample_rate);

    let mut next_value = move || {
        let mut rng = rand::thread_rng();
        let change: f32 = rng.gen_range(-0.2..0.2);
        current_value += change;
        current_value = current_value.clamp(-1.0, 1.0);

        // Fade in the brown noise sound
        let result = t * current_value;
        t = (t + 0.2 / sample_rate).min(1.0);

        lpf.apply(result)
    };

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            write_data(data, channels, &mut next_value)
        },
        err_fn,
        None,
    )?;
    Ok(stream)
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

struct LowPassFilter {
    cutoff: f32,
    sample_rate: f32,
    prev_output: f32,
}

impl LowPassFilter {
    fn new(cutoff: f32, sample_rate: f32) -> Self {
        LowPassFilter {
            cutoff,
            sample_rate,
            prev_output: 0.0,
        }
    }

    fn apply(&mut self, input: f32) -> f32 {
        let rc = 1.0 / (self.cutoff * 2.0 * std::f32::consts::PI);
        let dt = 1.0 / self.sample_rate;
        let alpha = dt / (rc + dt);

        let output = self.prev_output + alpha * (input - self.prev_output);

        self.prev_output = output;

        output
    }
}
