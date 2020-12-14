use cpal;
use anyhow::Error;
use cpal::traits::{DeviceTrait, HostTrait};
use std::sync::mpsc::*;
use std::slice;

extern crate piston_window;

use piston_window::*;


mod bindings;
#[link(name = "colorchord")] extern {} 

struct Note {
    color: u32,
    amplitude: f32,
}

fn main() {
    println!("Hello, world!");
    
    let mut window: PistonWindow =
    WindowSettings::new("Hello Piston!", [1400, 480])
    .exit_on_esc(true).build().unwrap();
    window.next();
    
    let (tx, rx) = channel();
    let mut r = ringbuffer::new(tx);
    //r.start();
    let host = cpal::default_host();
    
    let notefinder = unsafe{ bindings::notefinder::CreateNoteFinder(48000) };
    
    let device = host
    .default_input_device()
    .expect("Failed to get default input device");
    
    println!("{:?}", device.name());
    
    let config = &cpal::StreamConfig {
        channels: 1,
        buffer_size: cpal::BufferSize::Fixed(1024),
        sample_rate: cpal::SampleRate(48_000)
    };
    
    let err_fn = move |err| {
        eprintln!("an error occurred on stream: {}", err);
    };
    
    let stream = device.build_input_stream(config,
        move |data, inp: &cpal::InputCallbackInfo| r.audio_callback(data, inp),
        err_fn);
        
        while let Ok(v) = rx.recv() {
            unsafe {
                bindings::notefinder::RunNoteFinder(notefinder, v.as_ptr(), 0, 1024)
            }
            unsafe {
                //println!("{:?}", (*notefinder).freqbins);
                //println!("{:?}", (*notefinder).freqbins * (*notefinder).octaves);
                let notepeaks = (*notefinder).freqbins / 2;

                let mut bins = Vec::new();
                
                for l in  0..notepeaks {
                    if *(*notefinder).note_amplitudes_out.offset(l as isize) < 0.0 {
                        continue;
                    }
                    let note = *(*notefinder).note_positions.offset(l as isize) / (*notefinder).freqbins as f32;
                    //print!("{:?} ", note);
                    bins.push(Note{
                        amplitude: *(*notefinder).note_amplitudes_out.offset(l as isize),
                        color: bindings::color::CCtoHEX(note, 1.0, 1.0),
                    })
                }
                
                if let Some(event) = window.next() {
                    
                    
                    window.draw_2d(&event, |context, graphics, _device| {
                        clear([1.0; 4], graphics);

                        for (i, n) in bins.iter().enumerate() {
                           // print!("{:x} ", b.color);
                            let (b, g, r) = ( (n.color >> 16)&0xff, (n.color >> 8)&0xff, n.color&0xff );
                            let (r, g, b) = (r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0); 

                            rectangle([r, g, b, 1.0],
                                [((i * 100) + 10) as f64 , 0.0, 100.0, n.amplitude as f64 * 400.0],
                                context.transform,
                                graphics);
                            

                         }
                        });
                    }
                    
                    
                }
                
            }
            std::thread::sleep(std::time::Duration::from_secs(120));
            drop(stream);
            
            
        }
        
        
        pub struct ringbuffer {
            buffersize: usize,
            soundhead: usize,
            buffer: [f32; 8192],
            nf: Sender<Vec<f32>>,
        }
        
        impl ringbuffer {
            fn new(nf: Sender<Vec<f32>>) -> ringbuffer {
                return ringbuffer {
                    buffersize: 8192,
                    buffer: [0.0f32; 8192],
                    soundhead: 0,
                    nf,
                }
            }
            
            fn audio_callback(&mut self, input: &[f32], denis: &cpal::InputCallbackInfo) {
                self.buffer[self.soundhead .. self.soundhead + input.len()].copy_from_slice(input);
                self.soundhead += input.len();
                
                if self.soundhead >= self.buffersize {
                    self.soundhead = self.soundhead % self.buffersize;
                }
                
                self.nf.send(input.to_vec());
            }
        }