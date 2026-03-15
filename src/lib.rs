use std::slice;
mod internal;


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

/// Profiling timers from the internal C pipeline.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Timing {
    pub start: f64,
    pub dft: f64,
    pub filter: f64,
    pub decompose: f64,
    pub finalize: f64,
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

// SAFETY: The underlying C NoteFinder state is not thread-safe.
// Send is safe because ownership can transfer between threads.
// Sync is intentionally NOT implemented — concurrent access from
// multiple threads would race on the C state.
unsafe impl Send for Notefinder {}

impl Drop for Notefinder {
    fn drop(&mut self) {
        unsafe {
            let nf = &mut *self.nf;
            libc::free(nf.note_positions as *mut libc::c_void);
            libc::free(nf.note_amplitudes as *mut libc::c_void);
            libc::free(nf.note_amplitudes_out as *mut libc::c_void);
            libc::free(nf.note_amplitudes2 as *mut libc::c_void);
            libc::free(nf.note_founds as *mut libc::c_void);
            libc::free(nf.note_peaks_to_dists_mapping as *mut libc::c_void);
            libc::free(nf.enduring_note_id as *mut libc::c_void);
            libc::free(nf.frequencies as *mut libc::c_void);
            libc::free(nf.outbins as *mut libc::c_void);
            libc::free(nf.folded_bins as *mut libc::c_void);
            libc::free(nf.dists as *mut libc::c_void);
            libc::free(self.nf as *mut libc::c_void);
        }
    }
}

impl Notefinder {
    /// Create a new instance of the Notefinder with the desired samplerate.
    ///
    /// Samplerate can only be set during creation.
    pub fn new(samplerate: i32) -> Notefinder {
        Notefinder {
            nf: unsafe { internal::CreateNoteFinder(samplerate) },
        }
    }

    /// Run the notefinder over the provided buffer
    pub fn run(&mut self, data: &[f32]) {
        unsafe {
            internal::RunNoteFinder(self.nf, data.as_ptr(), 0, data.len() as i32);
        }
    }

    /// Get the discovered notes
    pub fn get_notes(&self) -> Vec<Note> {
        unsafe {
            let nf = &*self.nf;
            let note_peaks = nf.note_peaks as usize;
            let freqbins = nf.freqbins as f32;
            let positions = slice::from_raw_parts(nf.note_positions, note_peaks);
            let amps_out = slice::from_raw_parts(nf.note_amplitudes_out, note_peaks);
            let amps2 = slice::from_raw_parts(nf.note_amplitudes2, note_peaks);
            let enduring = slice::from_raw_parts(nf.enduring_note_id, note_peaks);
            let dists = slice::from_raw_parts(nf.dists, note_peaks);

            (0..note_peaks)
                .map(|i| Note {
                    active: amps_out[i] > 0.0,
                    id: positions[i] / freqbins,
                    dist: NoteDists {
                        amp: dists[i].amp,
                        mean: dists[i].mean,
                        sigma: dists[i].sigma,
                        taken: dists[i].taken != 0,
                    },
                    amplitude_out: amps_out[i],
                    amplitude_iir2: amps2[i],
                    endured: enduring[i],
                })
                .collect()
        }
    }

    /// Get the folded frequency bins
    pub fn get_folded(&self) -> &[f32] {
        unsafe { slice::from_raw_parts((*self.nf).folded_bins, (*self.nf).freqbins as usize) }
    }

    /// Get the raw DFT output bins (length = freqbins * octaves)
    pub fn get_outbins(&self) -> &[f32] {
        unsafe {
            let nf = &*self.nf;
            slice::from_raw_parts(nf.outbins, (nf.freqbins * nf.octaves) as usize)
        }
    }

    /// Get the frequency array (length = freqbins * octaves)
    pub fn get_frequencies(&self) -> &[f32] {
        unsafe {
            let nf = &*self.nf;
            slice::from_raw_parts(nf.frequencies, (nf.freqbins * nf.octaves) as usize)
        }
    }

    /// Get the raw distribution data
    pub fn get_distributions(&self) -> &[internal::NoteDists] {
        unsafe {
            let nf = &*self.nf;
            slice::from_raw_parts(nf.dists, nf.dists_count as usize)
        }
    }

    /// Number of note peaks tracked
    pub fn note_peaks(&self) -> usize {
        unsafe { (*self.nf).note_peaks as usize }
    }

    /// Number of frequency bins per octave
    pub fn frequency_bins(&self) -> i32 {
        unsafe { (*self.nf).freqbins }
    }

    /// Number of octaves
    pub fn octaves(&self) -> i32 {
        unsafe { (*self.nf).octaves }
    }

    /// Reciprocal of sample rate
    pub fn sample_rate(&self) -> f32 {
        unsafe { (*self.nf).sps_rec }
    }

    /// Get internal profiling timers from the last `run()` call
    pub fn timing(&self) -> Timing {
        unsafe {
            let nf = &*self.nf;
            Timing {
                start: nf.StartTime,
                dft: nf.DFTTime,
                filter: nf.FilterTime,
                decompose: nf.DecomposeTime,
                finalize: nf.FinalizeTime,
            }
        }
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
    note %= 1.0;
    note *= 12.0;
    let hue = if note < 4.0 {
        (4.0 - note) / 24.0
    } else if note < 8.0 {
        (4.0 - note) / 12.0
    } else {
        (12.0 - note) / 8.0 + 1.0 / 6.0
    };
    hsv_to_rgb(hue, saturation, value)
}

/// Convert HSV color values to RGB, matching the C colorchord `HSVtoHEX` algorithm.
///
/// Hue is in the range 0.0..1.0 (mapped to 0-360 degrees internally),
/// saturation and value are in the range 0.0..1.0.
pub fn hsv_to_rgb(hue: f32, saturation: f32, value: f32) -> [f32; 3] {
    let mut ro = (hue * 6.0).rem_euclid(6.0);
    ro = (ro + 1.0).rem_euclid(6.0); // 60° hue rotation matching C

    let (mut pr, mut pg, mut pb) = (0.0f32, 0.0f32, 0.0f32);
    if ro < 1.0 {
        pr = 1.0;
        pg = 1.0 - ro;
    } else if ro < 2.0 {
        pr = 1.0;
        pb = ro - 1.0;
    } else if ro < 3.0 {
        pr = 3.0 - ro;
        pb = 1.0;
    } else if ro < 4.0 {
        pb = 1.0;
        pg = ro - 3.0;
    } else if ro < 5.0 {
        pb = 5.0 - ro;
        pg = 1.0;
    } else {
        pg = 1.0;
        pr = ro - 5.0;
    }

    pr *= value;
    pg *= value;
    pb *= value;

    let avg = pr + pg + pb;
    pr = pr * saturation + avg * (1.0 - saturation);
    pg = pg * saturation + avg * (1.0 - saturation);
    pb = pb * saturation + avg * (1.0 - saturation);

    // Channel swap matching C: og=pb, ob=pg
    [pr.clamp(0.0, 1.0), pb.clamp(0.0, 1.0), pg.clamp(0.0, 1.0)]
}
