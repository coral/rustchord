use std::slice;
mod internal;

use palette::{FromColor, IntoColor, Lch, Saturate, Shade, Srgb, Hsv, Hue, Pixel};

pub struct NotefinderResult<'a> {
    pub freqbins: i32,
    pub notepeaks: i32,
    pub positions: &'a [f32],
    pub amplitudes: &'a [f32],
    pub folded: &'a [f32], 
    pub dists: Vec<NoteDists>
}
pub struct NoteDists {
    pub amp: f32,   //Amplitude of normal distribution
    pub mean: f32,  //Mean of normal distribution
    pub sigma: f32, //Sigma of normal distribution
    pub taken: bool //Is distribution associated with any notes?
}

pub struct Note {
    pub dist: NoteDists,
}

pub struct Notefinder {
    nf: *mut internal::NoteFinder,
}

unsafe impl Send for Notefinder {}

impl Notefinder {
    
    pub fn new(samplerate:i32) -> Notefinder {
        return Notefinder{
            nf: unsafe{internal::CreateNoteFinder(samplerate)},
        }
    }
    
    pub fn run(&mut self, data: &[f32]) {
        unsafe {
            internal::RunNoteFinder(self.nf, data.as_ptr(), 0, data.len() as i32);
        }
    }
    
    pub fn get_notes<'a> (&'a self) -> NotefinderResult<'a>  {
        let freqbins = unsafe{(*self.nf).freqbins};
        let notepeaks = freqbins / 2;
        
        return NotefinderResult{
            freqbins,
            notepeaks,
            positions:  unsafe{ slice::from_raw_parts((*self.nf).note_positions, freqbins as usize)},
            amplitudes: unsafe{ slice::from_raw_parts((*self.nf).note_amplitudes_out, notepeaks as usize)},
            folded: unsafe{ slice::from_raw_parts((*self.nf).folded_bins, freqbins as usize)},
            dists: unsafe{ (0..notepeaks).map(|i| { let raw = (*self.nf).dists.offset(i as isize);
                NoteDists { 
                    amp: (*raw).amp,
                    mean: (*raw).mean,
                    sigma: (*raw).sigma,
                    taken: (*raw).taken != 0,
                }
            }
        ).collect::<Vec<_>>()}
        
    }
}

pub fn get_folded<'a> (&'a self) -> &'a [f32]{
    return unsafe{ slice::from_raw_parts((*self.nf).folded_bins, (*self.nf).freqbins as usize)}; 
}

}

pub struct Color {
    
}

    pub fn cc_to_rgb(mut note: f32, saturation: f32, value: f32) -> [f32; 3] {
        
        let mut hue = 0.0;
        note %= 1.0;
        note *= 12.0;
        if note < 4.0 
        {
            //Needs to be YELLOW->RED
            hue = (4.0 - note) / 24.0;
        }
        else if note < 8.0 
        {
            //            [4]  [8]
            //Needs to be RED->BLUE
            hue = ( 4.0 - note ) / 12.0;
        }
        else
        {
            //             [8] [12]
            //Needs to be BLUE->YELLOW
            hue = ( 12.0 - note ) / 8.0 + 1.0/6.0;
        }
    
        let c:Hsv = Hsv::new(hue*360., saturation, value).into_color(); 
        Srgb::from_color(c).into_format().into_raw()
        
    }
    
