use std::slice;
mod internal;

use palette::{Hsv, IntoColor, Pixel, Srgb};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use thiserror::Error;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct NoteDists {
    /// Amplitude of normal distribution
    pub amp: f32,
    /// Mean of normal distribution
    pub mean: f32,
    /// Sigma of normal distribution
    pub sigma: f32,
    /// Is distribution associated with any notes?
    pub taken: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Note {
    pub active: bool,
    pub id: f32,
    pub dist: NoteDists,
    pub amplitude_out: f32,
    pub amplitude_iir2: f32,
    pub endured: i32,
}

#[derive(Error, Debug)]
pub enum NoteFinderValidationError<T: Debug> {
    #[error("Outside valid range ({expected_min:?} - {expected_max:?}, found {found:?})")]
    OutsideValidRange {
        expected_min: T,
        expected_max: T,
        found: T,
    },
    #[error("unknown notefinder error")]
    Unknown,
}

macro_rules! notefinder_configuration {
    (
    $(#[$meta:meta])*
    $func_name:ident, $setting:ident, $v:ty, $name:ident, $min:expr, $max:expr) => {
        $(#[$meta])*
        pub fn $func_name(&self, $name: $v) -> Result<(), NoteFinderValidationError<$v>> {
            if $name < $min || $name > $max {
                return Err(NoteFinderValidationError::OutsideValidRange {
                    expected_min: $min,
                    expected_max: $max,
                    found: $name,
                });
            }

            unsafe { (*self.nf).$setting = $name }
            Ok(())
        }
    };
}

pub enum DFTAlgorithm {
    /// Fastest algorithm, results are worse. Useful on low end hardware
    DFTQuick,
    /// Default algorithm
    DFTProgressive,
    /// Progressive DFT using integer math
    DFTProgressiveInteger,
    /// Progressive DFT using skippy integer math
    DFTProgressiveIntegerSkippy,
    /// Progressive DFT using float32
    DFTProgressive32,
}

pub struct Notefinder {
    nf: *mut internal::NoteFinder,
}

unsafe impl Send for Notefinder {}

impl Notefinder {
    /// Create a new instance of the Notefinder with the desired samplerate.
    ///
    /// Samplerate can only be set during creation.
    pub fn new(samplerate: i32) -> Notefinder {
        return Notefinder {
            nf: unsafe { internal::CreateNoteFinder(samplerate) },
        };
    }

    /// Run the notefinder over the provided buffer
    pub fn run(&mut self, data: &[f32]) {
        unsafe {
            internal::RunNoteFinder(self.nf, data.as_ptr(), 0, data.len() as i32);
        }
    }

    /// Get the discovered notes
    pub fn get_notes(&self) -> Vec<Note> {
        let freqbins: f32 = unsafe { (*self.nf).freqbins } as f32;
        let note_peaks: usize = unsafe { (*self.nf).note_peaks } as usize;
        let mut notes: Vec<Note> = Vec::new();
        for i in 0..note_peaks {
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

    /// Use this to change the Discrete Fourier transform algorithm.
    ///
    /// Options defined in DFTAlgorithm
    pub fn set_dft_algorithm(&mut self, algo: DFTAlgorithm) {
        use DFTAlgorithm::*;
        let dftalgo = match algo {
            DFTQuick => 0,
            DFTProgressive => 1,
            DFTProgressiveInteger => 2,
            DFTProgressiveIntegerSkippy => 3,
            DFTProgressive32 => 4,
        };
        unsafe { (*self.nf).do_progressive_dft = dftalgo }
    }
    notefinder_configuration!(
        /// Sets the span of octaves
        /// Defaults to 8
        set_octaves,
        octaves,
        i32,
        octaves,
        0,
        8
    );
    notefinder_configuration!(
        /// Defines the number of frequency bins
        /// Defaults to 24
        set_frequency_bins,
        freqbins,
        i32,
        frequency_bins,
        12,
        48
    );
    notefinder_configuration!(
        /// Set the base Hz for the notefinder to start at
        /// Defaults to 0
        set_base_hz,
        base_hz,
        f32,
        base_hz,
        0.,
        20000.
    );
    notefinder_configuration!(
        /// Controls the strength of the filter
        /// Defaults to 0.5
        set_filter_strength,
        filter_strength,
        f32,
        filter_strength,
        0.,
        1.
    );
    notefinder_configuration!(
        /// Set filter iterations, the higher the better but does cost CPU
        /// Defaults to 1.
        set_filter_iterations,
        filter_iter,
        i32,
        filter_iterations,
        1,
        8
    );
    notefinder_configuration!(
        /// Set decompose iterations, defaults to 1000
        set_decompose_iterations,
        decompose_iterations,
        i32,
        decompose_iterations,
        100,
        10000
    );
    notefinder_configuration!(
        /// Amplify input across the board
        set_amplification,
        amplify,
        f32,
        amplification,
        0.0,
        40.0
    );
    notefinder_configuration!(
        /// How much to compress the sound by before putting it into the compressor.
        set_compress_exponent,
        compress_exponenet,
        f32,
        compress_exponent,
        0.,
        10.
    );
    notefinder_configuration!(
        /// Exponent of the compressor lower = make more uniform.
        set_compress_coefficient,
        compress_coefficient,
        f32,
        compress_coefficient,
        0.,
        5.
    );
    notefinder_configuration!(
        /// At 300, there is still some minimal aliasing at higher frequencies.  Increase this for less low-end distortion
        /// Defaults to 300
        set_dft_speedup,
        dft_speedup,
        f32,
        dft_speedup,
        100.,
        20000.
    );
    notefinder_configuration!(
        /// The "tightness" of the curve, or how many samples back to look?
        /// Defaults to 16
        set_dft_q,
        dft_q,
        f32,
        dft_q,
        4.,
        64.
    );
    notefinder_configuration!(
        /// This controls the expected shape of the normal distributions.
        /// Defaults to 1.4
        ///
        /// Author of Colorchord notes "I am not sure how to calculate this from samplerate, Q and bins."
        set_default_sigma,
        default_sigma,
        f32,
        default_sigma,
        0.,
        8.
    );
    notefinder_configuration!(
        /// How far established notes are allowed to "jump" in order to attach themselves to a new "peak"
        /// Default 0.5
        set_note_jumpability,
        note_jumpability,
        f32,
        note_jumpability,
        0.,
        8.
    );
    notefinder_configuration!(
        /// How close established notes need to be to each other before they can be "combined" into a single note.
        /// Defaults to 0.5
        set_note_combine_distance,
        note_combine_distance,
        f32,
        note_combine_distance,
        0.,
        4.
    );
    notefinder_configuration!(set_slope, slope, f32, slope, 0., 1.);
    notefinder_configuration!(
        set_note_attach_freq_iir,
        note_attach_freq_iir,
        f32,
        note_attach_freq_iir,
        0.,
        3.
    );
    notefinder_configuration!(
        set_note_attach_amp_iir,
        note_attach_amp_iir,
        f32,
        note_attach_amp_iir,
        0.,
        3.
    );
    notefinder_configuration!(
        set_note_attach_amp_iir2,
        note_attach_amp_iir2,
        f32,
        note_attach_amp_iir2,
        0.,
        3.
    );
    notefinder_configuration!(
        /// A distribution must be /this/ big otherwise, it will be discarded.
        /// Defaults to 0.02
        set_note_minimum_new_distribution_value,
        note_minimum_new_distribution_value,
        f32,
        note_minimum_new_distribution_value,
        0.,
        1.
    );
    notefinder_configuration!(
        /// How much to decimate the output notes to reduce spurious noise
        set_note_out_chop,
        note_out_chop,
        f32,
        note_out_chop,
        0.,
        1.
    );
    notefinder_configuration!(
        /// IIR (infinite impulse response) to impose the output of the IIR.
        set_dft_iir,
        dft_iir,
        f32,
        dft_iir,
        0.,
        10.
    );
}

pub fn cc_to_rgb(mut note: f32, saturation: f32, value: f32) -> [f32; 3] {
    let hue: f32;
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

    let c: Srgb = Hsv::new(hue * 360., saturation, value).into_color();

    c.into_format().into_raw()
}
