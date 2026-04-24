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
    sample_rate: f32,
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
            sample_rate,
            decimation,
            decimation_counter: 0,
        }
    }

    /// Demoduliert IQ-Samples → Audio-Samples (f32, −1..1)
    pub fn process(&mut self, samples: &[Complex<f32>]) -> Vec<f32> {
        match self.mode {
            DemodMode::Am => self.demod_am(samples),
            DemodMode::Wbfm => self.demod_wbfm(samples),
        }
    }

    fn demod_am(&self, samples: &[Complex<f32>]) -> Vec<f32> {
        samples.iter()
            .step_by(self.decimation)
            .map(|s| s.norm().min(1.0))
            .collect()
    }

    fn demod_wbfm(&mut self, samples: &[Complex<f32>]) -> Vec<f32> {
        let mut audio = Vec::with_capacity(samples.len() / self.decimation + 1);
        for &s in samples {
            // FM-Diskriminator: Phasendifferenz aufeinanderfolgender Samples
            let demod = (s * self.prev_sample.conj()).arg() / std::f32::consts::PI;
            self.prev_sample = s;
            self.decimation_counter += 1;
            if self.decimation_counter >= self.decimation {
                self.decimation_counter = 0;
                audio.push(demod.clamp(-1.0, 1.0));
            }
        }
        audio
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_am_envelope() {
        // AM: konstante Amplitude 0.5 → Audio ~0.5
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
        // WBFM: konstante Phase → kein Phasensprung → Stille (~0)
        let samples: Vec<Complex<f32>> = (0..2048)
            .map(|_| Complex::new(1.0, 0.0))
            .collect();
        let mut demod = Demodulator::new(DemodMode::Wbfm, 2_048_000.0);
        let audio = demod.process(&samples);
        assert!(!audio.is_empty());
        let mean: f32 = audio.iter().sum::<f32>() / audio.len() as f32;
        assert!(mean.abs() < 0.05,
            "WBFM-Stille erwartet, got mean={}", mean);
    }

    #[test]
    fn test_audio_range() {
        // Audio-Ausgabe muss immer im Bereich −1..1 liegen
        let samples: Vec<Complex<f32>> = (0..4096)
            .map(|i| {
                let phase = 0.01 * i as f32;
                Complex::new(phase.cos(), phase.sin())
            })
            .collect();
        let mut demod = Demodulator::new(DemodMode::Wbfm, 2_048_000.0);
        let audio = demod.process(&samples);
        for &s in &audio {
            assert!(s >= -1.0 && s <= 1.0,
                "Audio außerhalb −1..1: {}", s);
        }
    }

    #[test]
    fn test_decimation_reduces_sample_count() {
        // Bei 2.048 MS/s und 44.1 kHz Audio → Dezimationsfaktor ~46
        let input_count = 4096;
        let samples: Vec<Complex<f32>> = vec![Complex::new(0.5, 0.0); input_count];
        let mut demod = Demodulator::new(DemodMode::Am, 2_048_000.0);
        let audio = demod.process(&samples);
        // Audio muss deutlich weniger Samples haben als Eingabe
        assert!(audio.len() < input_count / 10,
            "Dezimation zu schwach: {} Eingabe → {} Ausgabe", input_count, audio.len());
        assert!(!audio.is_empty(), "Audio darf nicht leer sein");
    }

    #[test]
    fn test_empty_input_returns_empty() {
        let mut demod = Demodulator::new(DemodMode::Wbfm, 2_048_000.0);
        let audio = demod.process(&[]);
        assert!(audio.is_empty());
    }
}
