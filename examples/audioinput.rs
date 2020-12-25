use cpal;
use cpal::traits::{DeviceTrait, HostTrait};
use rustchord::{self, cc_to_rgb};
use std::sync::mpsc::*;

use piston_window;

use piston_window::*;

fn main() {
    println!("Hello, world!");

    let mut window: PistonWindow = WindowSettings::new("colorchord binding demo", [1400, 480])
        .exit_on_esc(true)
        .build()
        .unwrap();
    window.next();
    let (tx, rx) = channel();
    let mut r = Ringbuffer::new(tx);
    let host = cpal::default_host();
    let mut notefinder = rustchord::Notefinder::new(48000);
    let device = host
        .default_input_device()
        .expect("Failed to get default input device");
    println!("{:?}", device.name());
    let config = &cpal::StreamConfig {
        channels: 1,
        buffer_size: cpal::BufferSize::Fixed(1024),
        sample_rate: cpal::SampleRate(48_000),
    };
    let err_fn = move |err| {
        eprintln!("an error occurred on stream: {}", err);
    };

    let stream = device.build_input_stream(
        config,
        move |data, inp: &cpal::InputCallbackInfo| r.audio_callback(data, inp),
        err_fn,
    );

    while let Ok(v) = rx.recv() {
        notefinder.run(&v);
        let notes = notefinder.get_notes();
        let res = notefinder.get_folded();

        let width = window.size().width / 24.;
        let notew = window.size().width / 12.;
        let middle = window.size().height / 2.;
        let bottom = window.size().height;

        if let Some(event) = window.next() {
            window.draw_2d(&event, |context, graphics, _device| {
                clear([1.0; 4], graphics);

                //Frequency bins
                for (i, n) in res.into_iter().enumerate() {
                    let c = cc_to_rgb((i as f32 + 0.5) / 24., 1.0, 1.0);

                    rectangle(
                        [c[0], c[1], c[2], 1.0],
                        [width * i as f64, bottom, width, -(*n as f64 * 800.)],
                        context.transform,
                        graphics,
                    );
                }

                for (i, n) in notes.into_iter().enumerate() {
                    if !n.active {
                        continue;
                    }
                    let c = cc_to_rgb(n.id as f32, 1.0, 1.0);

                    rectangle(
                        [c[0], c[1], c[2], 1.0],
                        [
                            notew * i as f64,
                            middle,
                            notew,
                            -(n.amplitude_out as f64 * 200.),
                        ],
                        context.transform,
                        graphics,
                    )
                }
            });
        }
    }
    std::thread::sleep(std::time::Duration::from_secs(120));
    drop(stream);
}

const BUFFER_SIZE: usize = 8096;
type Buffer = [f32; BUFFER_SIZE];

pub struct Ringbuffer {
    soundhead: usize,
    buffer: Buffer,
    nf: Sender<Vec<f32>>,
}

impl Ringbuffer {
    fn new(nf: Sender<Vec<f32>>) -> Ringbuffer {
        return Ringbuffer {
            buffer: [0.0f32; BUFFER_SIZE],
            soundhead: 0,
            nf,
        };
    }

    fn audio_callback(&mut self, mut input: &[f32], _info: &cpal::InputCallbackInfo) {
        if input.len() > self.buffer.len() {
            input = &input[..self.buffer.len()]
        }

        let old_head = self.soundhead;
        self.soundhead += input.len();

        if self.soundhead < self.buffer.len() {
            self.buffer[old_head..self.soundhead].copy_from_slice(input);
        } else {
            self.soundhead %= self.buffer.len();
            let first_len = self.buffer.len() - old_head;
            self.buffer[old_head..].copy_from_slice(&input[..first_len]);
            self.buffer[..self.soundhead].copy_from_slice(&input[first_len..]);
        }

        let mut out = Vec::with_capacity(self.buffer.len());
        out.extend_from_slice(&self.buffer[self.soundhead..]);
        out.extend_from_slice(&self.buffer[..self.soundhead]);

        let _ = self.nf.send(out);
    }
}
