use num_complex::Complex;
use rustfft::FftPlanner;
use std::sync::Arc;

pub const FFT_SIZE: usize = 1024;

/// Echtzeit-FFT-Prozessor. Pre-allokiert alle Buffer um Heap-Allokationen im DSP-Thread zu vermeiden.
pub struct FftProcessor {
    fft: Arc<dyn rustfft::Fft<f32>>,
    scratch: Vec<Complex<f32>>,
    window_buf: Vec<Complex<f32>>,  // wiederverwendeter Buffer für gefensterte Samples
    hann: Vec<f32>,                 // pre-computed Hann-Koeffizienten
}

// Statische Prüfung: FftProcessor muss Send sein (wird im DSP-Thread verwendet)
const _: () = {
    fn assert_send<T: Send>() {}
    fn _check() { assert_send::<FftProcessor>(); }
};

impl FftProcessor {
    pub fn new() -> Self {
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(FFT_SIZE);
        let scratch = vec![Complex::default(); fft.get_inplace_scratch_len()];
        let window_buf = vec![Complex::default(); FFT_SIZE];

        // Hann-Koeffizienten einmalig berechnen
        let hann: Vec<f32> = (0..FFT_SIZE)
            .map(|i| 0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32
                / (FFT_SIZE - 1) as f32).cos()))
            .collect();

        Self { fft, scratch, window_buf, hann }
    }

    /// Berechnet dBFS-Magnitude aus IQ-Samples mit Hann-Fenster und FFT-Shift.
    ///
    /// Eingabe:  FFT_SIZE komplexe Samples
    /// Ausgabe:  FFT_SIZE Magnitude-Werte in dBFS (−120..0), DC zentriert (Bin 512)
    pub fn process(&mut self, samples: &[Complex<f32>], out: &mut [f32]) {
        if samples.len() != FFT_SIZE || out.len() != FFT_SIZE {
            return; // Sicher ignorieren statt paniken
        }

        // Hann-Fenster anwenden (kein Heap-Alloc — window_buf wiederverwendet)
        for (i, (&s, &w)) in samples.iter().zip(self.hann.iter()).enumerate() {
            self.window_buf[i] = s * w;
        }

        self.fft.process_with_scratch(&mut self.window_buf, &mut self.scratch);

        // FFT-Shift + dBFS-Berechnung
        // Nach Shift: Bin 0 (DC) liegt bei Index FFT_SIZE/2 = 512
        let half = FFT_SIZE / 2;
        for i in 0..FFT_SIZE {
            let shifted = (i + half) % FFT_SIZE;
            let mag = self.window_buf[shifted].norm();
            out[i] = if mag > 0.0 {
                (20.0 * mag.log10() - 10.0 * (FFT_SIZE as f32).log10())
                    .max(-120.0)
                    .min(0.0)
            } else {
                -120.0
            };
        }
    }
}

impl Default for FftProcessor {
    fn default() -> Self { Self::new() }
}

/// Convenience-Funktion (C-ABI + Tests).
/// Verwendet Hann-Fenster und FFT-Shift — identisches Verhalten wie FftProcessor::process().
pub fn compute_fft_magnitude(samples: &[Complex<f32>], out: &mut [f32]) {
    FftProcessor::new().process(samples, out);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    /// Hilfsfunktion: komplexer Sinus bei gegebenem Bin
    fn sine_at_bin(bin: usize) -> Vec<Complex<f32>> {
        (0..FFT_SIZE)
            .map(|i| {
                let phase = 2.0 * PI * bin as f32 * i as f32 / FFT_SIZE as f32;
                Complex::new(phase.cos(), phase.sin())
            })
            .collect()
    }

    #[test]
    fn test_fft_processor_sine_peak() {
        // FftProcessor mit Hann-Fenster + FFT-Shift:
        // Sinus bei Bin 10 → Peak bei Index 512 + 10 = 522 (nach Shift)
        let samples = sine_at_bin(10);
        let mut out = vec![0.0f32; FFT_SIZE];
        FftProcessor::new().process(&samples, &mut out);

        let peak_idx = out.iter().enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .unwrap().0;

        // Hann-Fenster verteilt Energie auf Nachbarbins — Peak innerhalb ±2 Bins erlaubt
        let expected = FFT_SIZE / 2 + 10; // 522
        assert!(
            (peak_idx as isize - expected as isize).abs() <= 2,
            "Peak erwartet bei ~{}, got {}", expected, peak_idx
        );
        // Peak deutlich über Rauschen
        assert!(out[peak_idx] > out[0] + 20.0,
            "Peak-SNR zu gering: peak={:.1} noise={:.1}", out[peak_idx], out[0]);
    }

    #[test]
    fn test_fft_output_range() {
        // Alle Ausgabewerte müssen im dBFS-Bereich liegen
        let samples: Vec<Complex<f32>> = vec![Complex::new(0.5, 0.0); FFT_SIZE];
        let mut out = vec![0.0f32; FFT_SIZE];
        FftProcessor::new().process(&samples, &mut out);
        for &v in &out {
            assert!(v >= -120.0 && v <= 0.0,
                "dBFS-Wert außerhalb −120..0: {}", v);
        }
    }

    #[test]
    fn test_zero_input_returns_floor() {
        // Null-Signal → alle Ausgaben am Rauschboden (−120 dBFS)
        let samples = vec![Complex::new(0.0f32, 0.0); FFT_SIZE];
        let mut out = vec![0.0f32; FFT_SIZE];
        FftProcessor::new().process(&samples, &mut out);
        for &v in &out {
            assert_eq!(v, -120.0, "Null-Signal muss −120 dBFS ergeben, got {}", v);
        }
    }

    #[test]
    fn test_full_scale_stays_below_zero() {
        // Vollaussteuerung (Amplitude 1.0) → darf 0 dBFS nicht überschreiten
        let samples = vec![Complex::new(1.0f32, 0.0); FFT_SIZE];
        let mut out = vec![0.0f32; FFT_SIZE];
        FftProcessor::new().process(&samples, &mut out);
        for &v in &out {
            assert!(v <= 0.0, "Vollaussteuerung darf 0 dBFS nicht überschreiten, got {}", v);
        }
    }

    #[test]
    fn test_wrong_slice_length_does_not_panic() {
        // Falsche Slice-Länge → kein Panic, keine Ausgabe
        let samples = vec![Complex::new(1.0f32, 0.0); 512]; // zu kurz
        let mut out = vec![0.0f32; FFT_SIZE];
        FftProcessor::new().process(&samples, &mut out); // darf nicht paniken
    }

    #[test]
    fn test_compute_fft_magnitude_delegates_correctly() {
        // compute_fft_magnitude und FftProcessor::process liefern identische Ausgabe
        let samples = sine_at_bin(50);
        let mut out1 = vec![0.0f32; FFT_SIZE];
        let mut out2 = vec![0.0f32; FFT_SIZE];
        FftProcessor::new().process(&samples, &mut out1);
        compute_fft_magnitude(&samples, &mut out2);
        for (a, b) in out1.iter().zip(out2.iter()) {
            assert!((a - b).abs() < 1e-4, "Ausgaben weichen ab: {} vs {}", a, b);
        }
    }
}
