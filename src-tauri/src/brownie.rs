use crate::lowpass::LowPassFilter;
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    FromSample, Sample, SizedSample, Stream,
};
use rand::prelude::*;
use std::{
    sync::{
        atomic::{self, AtomicI8, Ordering},
        mpsc, Arc,
    },
    thread,
};
use tauri::Pixel;

pub enum Command {
    Mute,
    Unmute,
}

pub struct Brownie {
    sender: mpsc::Sender<Command>,
    muted: Arc<AtomicI8>,
}

impl Brownie {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel::<Command>();
        let muted = Arc::new(AtomicI8::new(1));
        let muted_local = Arc::clone(&muted);
        thread::spawn(move || {
            let _stream = create_stream(muted.clone()).unwrap();
            while let Ok(command) = receiver.recv() {
                match command {
                    Command::Mute => {
                        muted.store(-1, Ordering::Relaxed);
                    }
                    Command::Unmute => {
                        muted.store(1, Ordering::Relaxed);
                    }
                }
            }
        });

        Brownie {
            sender,
            muted: muted_local,
        }
    }

    pub fn mute(&self) {
        self.sender.send(Command::Mute).unwrap();
    }

    pub fn unmute(&self) {
        self.sender.send(Command::Unmute).unwrap();
    }

    pub fn is_muted(&self) -> bool {
        self.muted.load(atomic::Ordering::Relaxed) < 1
    }
}

fn create_stream(volume: Arc<AtomicI8>) -> anyhow::Result<Stream> {
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
        cpal::SampleFormat::I8 => run::<i8>(&device, &config.into(), volume),
        cpal::SampleFormat::I16 => run::<i16>(&device, &config.into(), volume),
        cpal::SampleFormat::I32 => run::<i32>(&device, &config.into(), volume),
        cpal::SampleFormat::I64 => run::<i64>(&device, &config.into(), volume),
        cpal::SampleFormat::U8 => run::<u8>(&device, &config.into(), volume),
        cpal::SampleFormat::U16 => run::<u16>(&device, &config.into(), volume),
        cpal::SampleFormat::U32 => run::<u32>(&device, &config.into(), volume),
        cpal::SampleFormat::U64 => run::<u64>(&device, &config.into(), volume),
        cpal::SampleFormat::F32 => run::<f32>(&device, &config.into(), volume),
        cpal::SampleFormat::F64 => run::<f64>(&device, &config.into(), volume),
        sample_format => panic!("Unsupported sample format '{sample_format}'"),
    }
}

fn run<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    volume: Arc<AtomicI8>,
) -> Result<Stream, anyhow::Error>
where
    T: SizedSample + FromSample<f32>,
{
    let sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;
    let mut current_value = 0.0;
    let mut t = 0.0;
    let mut lpf = LowPassFilter::new(40.0, sample_rate);

    let mut next_value = move || {
        let mut rng = rand::thread_rng();
        let change: f32 = rng.gen_range(-0.2..0.2);
        current_value += change;
        current_value = current_value.clamp(-1.0, 1.0);

        // Fade in the brown noise sound
        let result = t * current_value;
        let mag = 2.0 * volume.load(Ordering::Relaxed).cast::<f32>();
        t = (t + mag / sample_rate).clamp(0.0, 1.0);

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
    stream.play()?;
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
