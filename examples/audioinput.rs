use cpal;
use anyhow::Error;
use cpal::traits::{DeviceTrait, HostTrait};
use std::sync::mpsc::*;
use std::slice;
use rustchord::{self, cc_to_rgb};

use piston_window;

use piston_window::*;

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
    let host = cpal::default_host();
    
    let mut notefinder = rustchord::Notefinder::new(48000);
    
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
            notefinder.run(&v);
            let res = notefinder.get_folded();

            let width = window.size().width / 24.;
            let bottom = window.size().height / 2.;
            
            if let Some(event) = window.next() {

                window.draw_2d(&event, |context, graphics, _device| {
                    clear([1.0; 4], graphics);

                    //Frequency bins
                    for (i, n) in res.into_iter().enumerate() {

                        let c = cc_to_rgb((i as f32 + 0.5) / 24., 1.0, 1.0);

                        rectangle([c[0], c[1], c[2], 1.0],
                            [width * i as f64, bottom, width, -(*n as f64 * 800.)],
                            context.transform,
                            graphics);

                    }

                });
            }

            // for n in 0..res.notepeaks {
            //     if res.amplitudes[n as usize] < 0.0 {
            //         continue;
            //     }
            //     //print!("{:?} ", res.positions[n as usize]/res.freqbins as f32);

            // }

            // unsafe {
            //     rustchord::RunNoteFinder(notefinder, v.as_ptr(), 0, 1024)
            // }
            // unsafe {
            //     //println!("{:?}", (*notefinder).freqbins);
            //     //println!("{:?}", (*notefinder).freqbins * (*notefinder).octaves);
            //     let notepeaks = (*notefinder).freqbins / 2;

            //     let mut bins = Vec::new();
                
            //     for l in  0..notepeaks {
            //         if *(*notefinder).note_amplitudes_out.offset(l as isize) < 0.0 {
            //             continue;
            //         }
            //         let note = *(*notefinder).note_positions.offset(l as isize) / (*notefinder).freqbins as f32;
            //         //print!("{:?} ", note);
            //         // bins.push(Note{
            //         //     amplitude: *(*notefinder).note_amplitudes_out.offset(l as isize),
            //         //     color: rustchord::CCtoHEX(note, 1.0, 1.0),
            //         // })
            //     }
                
                // if let Some(event) = window.next() {
                    
                    
                //     window.draw_2d(&event, |context, graphics, _device| {a
                //         clear([1.0; 4], graphics);

                //         for (i, n) in bins.iter().enumerate() {
                //            // print!("{:x} ", b.color);
                //             let (b, g, r) = ( (n.color >> 16)&0xff, (n.color >> 8)&0xff, n.color&0xff );
                //             let (r, g, b) = (r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0); 

                            

                //          }
                //         });
                //     }
                    
                    
                // }
                
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