use std::sync::{Arc, Mutex};
use std::thread;
use num_complex::Complex;
use ringbuf::{traits::*, HeapProd, HeapRb};

use crate::device::{DeviceInfo, SdrDevice};
use crate::fft::{FftProcessor, FFT_SIZE};
use crate::demod::{Demodulator, DemodMode};
use crate::audio::AudioOutput;

const SAMPLE_RATE: f64 = 2_048_000.0;
const AUDIO_RATE: u32 = 44_100;

/// Shared state zwischen Empfänger-Thread und Swift-UI-Thread
struct SharedState {
    fft_data: [f32; FFT_SIZE],
    is_running: bool,
}

pub struct SdrappCore {
    state: Arc<Mutex<SharedState>>,
    iq_producer: Option<HeapProd<Complex<f32>>>,
    device_args: Option<String>,
    frequency_hz: u64,
    gain_db: f64,
    demod_mode: DemodMode,
}

impl SdrappCore {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(SharedState {
                fft_data: [-120.0; FFT_SIZE],
                is_running: false,
            })),
            iq_producer: None,
            device_args: None,
            frequency_hz: 100_000_000, // 100 MHz
            gain_db: 30.0,
            demod_mode: DemodMode::Wbfm,
        }
    }

    pub fn list_devices() -> Vec<DeviceInfo> {
        SdrDevice::enumerate()
    }

    pub fn set_device(&mut self, args: &str) {
        self.device_args = Some(args.to_string());
    }

    pub fn set_frequency(&mut self, hz: u64) {
        self.frequency_hz = hz;
    }

    pub fn set_gain(&mut self, db: f64) {
        self.gain_db = db;
    }

    pub fn set_demod(&mut self, mode: DemodMode) {
        self.demod_mode = mode;
    }

    /// Kopiert aktuelle FFT-Daten in out_buf. Gibt FFT_SIZE zurück.
    pub fn get_fft(&self, out_buf: &mut [f32]) -> usize {
        let state = self.state.lock().unwrap();
        let len = out_buf.len().min(FFT_SIZE);
        out_buf[..len].copy_from_slice(&state.fft_data[..len]);
        len
    }

    /// Startet Empfänger- und DSP-Thread.
    /// Note: The receiver thread (SoapySDR → ring buffer) is a Phase 1 stub.
    /// The DSP thread (ring buffer → FFT → demod → audio) is implemented.
    /// Actual hardware reading is completed in Task 15.
    pub fn start(&mut self) -> bool {
        let device_args = match &self.device_args {
            Some(a) => a.clone(),
            None => return false,
        };

        let mut device = match SdrDevice::open(&device_args) {
            Ok(d) => d,
            Err(e) => { eprintln!("Gerät öffnen fehlgeschlagen: {}", e); return false; }
        };

        if let Err(e) = device.configure(self.frequency_hz, self.gain_db, SAMPLE_RATE) {
            eprintln!("Gerät konfigurieren fehlgeschlagen: {}", e);
            return false;
        }

        let rb = HeapRb::<Complex<f32>>::new(SAMPLE_RATE as usize); // 1s Buffer
        let (iq_producer, mut iq_consumer) = rb.split();
        self.iq_producer = Some(iq_producer);

        let state = Arc::clone(&self.state);
        let demod_mode = self.demod_mode;

        // DSP-Thread: liest IQ aus Ring Buffer, rechnet FFT + Demod
        thread::spawn(move || {
            let mut fft = FftProcessor::new();
            let mut demod = Demodulator::new(demod_mode, SAMPLE_RATE as f32);
            let mut audio = match AudioOutput::new(AUDIO_RATE) {
                Ok(a) => a,
                Err(e) => { eprintln!("Audio-Init fehlgeschlagen: {}", e); return; }
            };
            let mut buf = vec![Complex::default(); FFT_SIZE];
            let mut fft_out = vec![0.0f32; FFT_SIZE];

            loop {
                {
                    let s = state.lock().unwrap();
                    if !s.is_running { break; }
                }

                let available = iq_consumer.occupied_len();
                if available < FFT_SIZE {
                    thread::sleep(std::time::Duration::from_millis(1));
                    continue;
                }

                for s in buf.iter_mut() {
                    *s = iq_consumer.try_pop().unwrap_or_default();
                }

                fft.process(&buf, &mut fft_out);
                {
                    let mut s = state.lock().unwrap();
                    s.fft_data.copy_from_slice(&fft_out);
                }

                let audio_samples = demod.process(&buf);
                audio.push_samples(&audio_samples);
            }
        });

        {
            let mut state_guard = self.state.lock().unwrap();
            state_guard.is_running = true;
        }

        true
    }

    pub fn stop(&mut self) {
        let mut state = self.state.lock().unwrap();
        state.is_running = false;
        self.iq_producer = None;
    }
}

impl Default for SdrappCore {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_core_default_state() {
        let core = SdrappCore::new();
        let mut buf = vec![0.0f32; FFT_SIZE];
        let len = core.get_fft(&mut buf);
        assert_eq!(len, FFT_SIZE);
        assert!(buf.iter().all(|&v| v <= -119.0));
    }

    #[test]
    fn test_list_devices_no_panic() {
        let devices = SdrappCore::list_devices();
        println!("Gefundene Geräte: {}", devices.len());
    }
}
