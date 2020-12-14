use std::slice;
mod internal;

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

pub struct Notefinder {
    nf: *mut internal::NoteFinder
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
    
    pub fn result<'a> (&'a self) -> NotefinderResult<'a>  {
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
    
}

// pub struct Color {

// }

// impl Color {
//     pub fn cc_to_hex() {
//         let (b, g, r) = ( (n.color >> 16)&0xff, (n.color >> 8)&0xff, n.color&0xff );
//         let (r, g, b) = (r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0); 
//     } 
// }