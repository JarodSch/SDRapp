use num_complex::Complex;
use rustfft::FftPlanner;
use std::sync::Arc;

pub const FFT_SIZE: usize = 1024;

pub struct FftProcessor {
    fft: Arc<dyn rustfft::Fft<f32>>,
    scratch: Vec<Complex<f32>>,
}

impl FftProcessor {
    pub fn new() -> Self {
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(FFT_SIZE);
        let scratch = vec![Complex::default(); fft.get_inplace_scratch_len()];
        Self { fft, scratch }
    }

    pub fn process(&mut self, samples: &[Complex<f32>], out: &mut [f32]) {
        assert_eq!(samples.len(), FFT_SIZE);
        assert_eq!(out.len(), FFT_SIZE);

        let mut buf: Vec<Complex<f32>> = samples
            .iter()
            .enumerate()
            .map(|(i, &s)| {
                // Hann-Fenster
                let w = 0.5
                    * (1.0
                        - (2.0 * std::f32::consts::PI * i as f32
                            / (FFT_SIZE - 1) as f32)
                            .cos());
                s * w
            })
            .collect();

        self.fft.process_with_scratch(&mut buf, &mut self.scratch);

        // Kein FFT-Shift: direkte Magnitude-Berechnung.
        // Ein komplexer Sinus bei Bin k liefert den Peak direkt bei buf[k].
        // FFT-Shift (für Visualisierung) kann im Swift-Layer erfolgen.
        for i in 0..FFT_SIZE {
            let mag = buf[i].norm();
            out[i] = if mag > 0.0 {
                20.0 * mag.log10() - 10.0 * (FFT_SIZE as f32).log10()
            } else {
                -120.0
            };
            out[i] = out[i].max(-120.0).min(0.0);
        }
    }
}

/// Convenience-Funktion für Tests und C-ABI.
/// Verwendet kein Fenster (Rechteck-Fenster), damit Bin-genaue Peaks
/// exakt bei ihrem Bin-Index landen.
pub fn compute_fft_magnitude(samples: &[Complex<f32>], out: &mut [f32]) {
    assert_eq!(samples.len(), FFT_SIZE);
    assert_eq!(out.len(), FFT_SIZE);

    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(FFT_SIZE);
    let mut scratch = vec![Complex::default(); fft.get_inplace_scratch_len()];
    let mut buf: Vec<Complex<f32>> = samples.to_vec();

    fft.process_with_scratch(&mut buf, &mut scratch);

    for i in 0..FFT_SIZE {
        let mag = buf[i].norm();
        out[i] = if mag > 0.0 {
            20.0 * mag.log10() - 10.0 * (FFT_SIZE as f32).log10()
        } else {
            -120.0
        };
        out[i] = out[i].max(-120.0).min(0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_fft_sine_peak() {
        let samples: Vec<Complex<f32>> = (0..FFT_SIZE)
            .map(|i| {
                let phase = 2.0 * PI * 10.0 * i as f32 / FFT_SIZE as f32;
                Complex::new(phase.cos(), phase.sin())
            })
            .collect();
        let mut magnitude = vec![0.0f32; FFT_SIZE];
        compute_fft_magnitude(&samples, &mut magnitude);
        let peak_idx = magnitude
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .unwrap()
            .0;
        assert_eq!(peak_idx, 10);
        assert!(
            magnitude[10] > magnitude[5] + 20.0,
            "Peak bei Bin 10 erwartet, got peak={} noise={}",
            magnitude[10],
            magnitude[5]
        );
    }

    #[test]
    fn test_fft_output_range() {
        let samples: Vec<Complex<f32>> = vec![Complex::new(0.5, 0.0); FFT_SIZE];
        let mut magnitude = vec![0.0f32; FFT_SIZE];
        compute_fft_magnitude(&samples, &mut magnitude);
        for &v in &magnitude {
            assert!(v > -200.0 && v < 10.0, "Unerwarteter dBm-Wert: {}", v);
        }
    }
}
