use num_complex::Complex;

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u32)]
pub enum DemodMode {
    Am = 0,
    Wbfm = 1,
}

pub struct Demodulator {
    mode: DemodMode,
    prev_sample: Complex<f32>,
    decimation: usize,
    decimation_counter: usize,
}

impl Demodulator {
    pub fn new(mode: DemodMode, sample_rate: f32) -> Self {
        let audio_rate = 44_100.0_f32;
        let decimation = ((sample_rate / audio_rate).round() as usize).max(1);
        Self {
            mode,
            prev_sample: Complex::new(1.0, 0.0),
            decimation,
            decimation_counter: 0,
        }
    }

    /// Demoduliert in caller-provided Buffer. Kein Heap-Alloc nach erstem Aufruf.
    pub fn process_into(&mut self, samples: &[Complex<f32>], out: &mut Vec<f32>) {
        out.clear();
        match self.mode {
            DemodMode::Am => self.demod_am_into(samples, out),
            DemodMode::Wbfm => self.demod_wbfm_into(samples, out),
        }
    }

    /// Convenience-Variante für Tests (allokiert intern).
    pub fn process(&mut self, samples: &[Complex<f32>]) -> Vec<f32> {
        let mut out = Vec::new();
        self.process_into(samples, &mut out);
        out
    }

    fn demod_am_into(&mut self, samples: &[Complex<f32>], out: &mut Vec<f32>) {
        for &s in samples {
            self.decimation_counter += 1;
            if self.decimation_counter >= self.decimation {
                self.decimation_counter = 0;
                out.push(s.norm().min(1.0));
            }
        }
    }

    fn demod_wbfm_into(&mut self, samples: &[Complex<f32>], out: &mut Vec<f32>) {
        for &s in samples {
            let demod = (s * self.prev_sample.conj()).arg() / std::f32::consts::PI;
            self.prev_sample = s;
            self.decimation_counter += 1;
            if self.decimation_counter >= self.decimation {
                self.decimation_counter = 0;
                out.push(demod.clamp(-1.0, 1.0));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_am_envelope() {
        let samples: Vec<Complex<f32>> = (0..2048)
            .map(|_| Complex::new(0.5, 0.0))
            .collect();
        let mut demod = Demodulator::new(DemodMode::Am, 2_048_000.0);
        let audio = demod.process(&samples);
        assert!(!audio.is_empty());
        for &s in &audio {
            assert!((s - 0.5).abs() < 0.01,
                "AM Hüllkurve falsch: erwartet ~0.5, got {}", s);
        }
    }

    #[test]
    fn test_wbfm_silence() {
        let samples: Vec<Complex<f32>> = (0..2048)
            .map(|_| Complex::new(1.0, 0.0))
            .collect();
        let mut demod = Demodulator::new(DemodMode::Wbfm, 2_048_000.0);
        let audio = demod.process(&samples);
        assert!(!audio.is_empty());
        let mean: f32 = audio.iter().sum::<f32>() / audio.len() as f32;
        assert!(mean.abs() < 0.05, "WBFM-Stille erwartet, got mean={}", mean);
    }

    #[test]
    fn test_audio_range() {
        let samples: Vec<Complex<f32>> = (0..4096)
            .map(|i| {
                let phase = 0.01 * i as f32;
                Complex::new(phase.cos(), phase.sin())
            })
            .collect();
        let mut demod = Demodulator::new(DemodMode::Wbfm, 2_048_000.0);
        let audio = demod.process(&samples);
        for &s in &audio {
            assert!(s >= -1.0 && s <= 1.0, "Audio außerhalb −1..1: {}", s);
        }
    }

    #[test]
    fn test_decimation_reduces_sample_count() {
        let input_count = 4096;
        let samples: Vec<Complex<f32>> = vec![Complex::new(0.5, 0.0); input_count];
        let mut demod = Demodulator::new(DemodMode::Am, 2_048_000.0);
        let audio = demod.process(&samples);
        assert!(audio.len() < input_count / 10,
            "Dezimation zu schwach: {} → {}", input_count, audio.len());
        assert!(!audio.is_empty());
    }

    #[test]
    fn test_empty_input_returns_empty() {
        let mut demod = Demodulator::new(DemodMode::Wbfm, 2_048_000.0);
        assert!(demod.process(&[]).is_empty());
    }

    #[test]
    fn test_am_continuous_across_calls() {
        // AM-Dezimation muss über Aufrufgrenzen hinweg kontinuierlich sein
        let chunk: Vec<Complex<f32>> = vec![Complex::new(0.5, 0.0); 2048];
        let mut demod = Demodulator::new(DemodMode::Am, 2_048_000.0);
        let call1 = demod.process(&chunk);
        let call2 = demod.process(&chunk);
        // Beide Chunks haben dieselbe Länge (±1 wegen Grenzfall)
        assert!((call1.len() as isize - call2.len() as isize).abs() <= 1,
            "AM-Dezimation nicht kontinuierlich: {} vs {}", call1.len(), call2.len());
    }

    #[test]
    fn test_wbfm_zero_samples_no_nan() {
        let mut demod = Demodulator::new(DemodMode::Wbfm, 2_048_000.0);
        let silence = vec![Complex::new(0.0_f32, 0.0); 2048];
        let audio = demod.process(&silence);
        for &s in &audio {
            assert!(!s.is_nan(), "NaN in WBFM output");
            assert!(s.is_finite(), "Inf in WBFM output");
        }
    }

    #[test]
    fn test_process_into_no_realloc() {
        // process_into soll bei wiederholten Aufrufen keinen Speicher allokieren
        let samples: Vec<Complex<f32>> = vec![Complex::new(0.5, 0.0); 2048];
        let mut demod = Demodulator::new(DemodMode::Am, 2_048_000.0);
        let mut out = Vec::new();
        demod.process_into(&samples, &mut out);
        let capacity_after_first = out.capacity();
        demod.process_into(&samples, &mut out);
        // Kapazität darf nach dem zweiten Aufruf nicht wachsen (kein Realloc)
        assert_eq!(out.capacity(), capacity_after_first,
            "process_into hat re-allokiert: {} → {}", capacity_after_first, out.capacity());
    }
}
