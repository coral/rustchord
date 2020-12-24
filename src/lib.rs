use std::slice;
mod internal;

use palette::{FromColor, Hsv, IntoColor, Pixel, Srgb};

pub struct NotefinderResult<'a> {
    pub freqbins: i32,
    pub notepeaks: i32,
    pub positions: &'a [f32],
    pub amplitudes: &'a [f32],
    pub folded: &'a [f32],
    pub dists: Vec<NoteDists>,
}

#[derive(Debug)]
pub struct NoteDists {
    pub amp: f32,    //Amplitude of normal distribution
    pub mean: f32,   //Mean of normal distribution
    pub sigma: f32,  //Sigma of normal distribution
    pub taken: bool, //Is distribution associated with any notes?
}

#[derive(Debug)]
pub struct Note {
    pub active: bool,
    pub id: f32,
    pub dist: NoteDists,
    pub amplitude_out: f32,
    pub amplitude_iir2: f32,
    pub endured: i32,
}

pub struct Notefinder {
    nf: *mut internal::NoteFinder,
}

unsafe impl Send for Notefinder {}

impl Notefinder {
    pub fn new(samplerate: i32) -> Notefinder {
        return Notefinder {
            nf: unsafe { internal::CreateNoteFinder(samplerate) },
        };
    }
    pub fn run(&mut self, data: &[f32]) {
        //dbg!(data);
        unsafe {
            internal::RunNoteFinder(self.nf, data.as_ptr(), 0, data.len() as i32);
        }
    }

    // pub fn get_notes_old<'a>(&'a self) -> NotefinderResult<'a> {
    //     let freqbins = unsafe { (*self.nf).freqbins };
    //     let notepeaks = freqbins / 2;

    //     return NotefinderResult {
    //         freqbins,
    //         notepeaks,
    //         positions: unsafe {
    //             slice::from_raw_parts((*self.nf).note_positions, freqbins as usize)
    //         },
    //         amplitudes: unsafe {
    //             slice::from_raw_parts((*self.nf).note_amplitudes_out, notepeaks as usize)
    //         },
    //         folded: unsafe { slice::from_raw_parts((*self.nf).folded_bins, freqbins as usize) },
    //         dists: unsafe {
    //             (0..notepeaks)
    //                 .map(|i| {
    //                     let raw = (*self.nf).dists.offset(i as isize);
    //                     NoteDists {
    //                         amp: (*raw).amp,
    //                         mean: (*raw).mean,
    //                         sigma: (*raw).sigma,
    //                         taken: (*raw).taken != 0,
    //                     }
    //                 })
    //                 .collect::<Vec<_>>()
    //         },
    //     };
    // }

    pub fn get_notes(&self) -> Vec<Note> {
        let freqbins: f32 = unsafe { (*self.nf).freqbins } as f32;
        let mut notes: Vec<Note> = Vec::new();
        for i in 0..12 {
            unsafe {
                let dist = (*self.nf).dists.offset(i as isize);
                notes.push(Note {
                    active: *(*self.nf).note_amplitudes_out.offset(i as isize) > 0.,
                    id: *(*self.nf).note_positions.offset(i as isize) / freqbins,
                    dist: NoteDists {
                        amp: (*dist).amp,
                        mean: (*dist).mean,
                        sigma: (*dist).sigma,
                        taken: (*dist).taken != 0,
                    },
                    amplitude_out: { *(*self.nf).note_amplitudes_out.offset(i as isize) },
                    amplitude_iir2: { *(*self.nf).note_amplitudes2.offset(i as isize) },
                    endured: { *(*self.nf).enduring_note_id.offset(i as isize) },
                })
            }
        }

        notes
    }

    pub fn get_folded<'a>(&'a self) -> &'a [f32] {
        return unsafe {
            slice::from_raw_parts((*self.nf).folded_bins, (*self.nf).freqbins as usize)
        };
    }
}

pub struct Color {}

pub fn cc_to_rgb(mut note: f32, saturation: f32, value: f32) -> [f32; 3] {
    let mut hue = 0.0;
    note %= 1.0;
    note *= 12.0;
    if note < 4.0 {
        //Needs to be YELLOW->RED
        hue = (4.0 - note) / 24.0;
    } else if note < 8.0 {
        //            [4]  [8]
        //Needs to be RED->BLUE
        hue = (4.0 - note) / 12.0;
    } else {
        //             [8] [12]
        //Needs to be BLUE->YELLOW
        hue = (12.0 - note) / 8.0 + 1.0 / 6.0;
    }

    let c: Hsv = Hsv::new(hue * 360., saturation, value).into_color();
    Srgb::from_color(c).into_format().into_raw()
}
