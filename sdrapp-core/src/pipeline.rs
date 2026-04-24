use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use num_complex::Complex;
use ringbuf::{traits::*, HeapRb};
use soapysdr::Direction::Rx;

use crate::device::{DeviceInfo, GainElementInfo, SdrDevice};
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
    /// Geteilter Zugriff auf das laufende SoapySDR-Gerät für Live-Tuning
    live_device: Arc<Mutex<Option<soapysdr::Device>>>,
    device_args: Option<String>,
    frequency_hz: u64,
    gain_db: f64,
    gain_elements: HashMap<String, f64>,
    demod_mode: DemodMode,
}

impl SdrappCore {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(SharedState {
                fft_data: [-120.0; FFT_SIZE],
                is_running: false,
            })),
            live_device: Arc::new(Mutex::new(None)),
            device_args: None,
            frequency_hz: 100_000_000, // 100 MHz
            gain_db: 30.0,
            gain_elements: HashMap::new(),
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
        // Live-Tuning: Frequenz direkt am laufenden Gerät setzen
        if let Ok(guard) = self.live_device.lock() {
            if let Some(dev) = guard.as_ref() {
                let _ = dev.set_frequency(Rx, 0, hz as f64, ());
            }
        }
    }

    pub fn set_gain(&mut self, db: f64) {
        self.gain_db = db;
        if let Ok(guard) = self.live_device.lock() {
            if let Some(dev) = guard.as_ref() {
                let _ = dev.set_gain(Rx, 0, db);
            }
        }
    }

    /// Listet Gain-Elemente des ausgewählten Geräts (öffnet kurz und schließt wieder).
    pub fn list_gain_elements(&self) -> Vec<GainElementInfo> {
        let args = match &self.device_args {
            Some(a) => a.clone(),
            None => return vec![],
        };
        // Falls Gerät läuft, direkt aus live_device lesen
        if let Ok(guard) = self.live_device.lock() {
            if let Some(dev) = guard.as_ref() {
                let names = dev.list_gains(Rx, 0).unwrap_or_default();
                return names.into_iter().map(|name| {
                    let range = dev.gain_element_range(Rx, 0, name.as_bytes())
                        .unwrap_or(soapysdr::Range { minimum: 0.0, maximum: 0.0, step: 1.0 });
                    let current = dev.gain_element(Rx, 0, name.as_bytes()).unwrap_or(0.0);
                    GainElementInfo {
                        name,
                        min_db: range.minimum,
                        max_db: range.maximum,
                        step_db: if range.step > 0.0 { range.step } else { 1.0 },
                        current_db: current,
                    }
                }).collect();
            }
        }
        // Gerät kurz öffnen nur zum Abfragen
        match SdrDevice::open(&args) {
            Ok(d) => d.list_gain_elements(),
            Err(_) => vec![],
        }
    }

    /// Setzt ein einzelnes Gain-Element live und speichert für nächsten Start.
    pub fn set_gain_element(&mut self, name: &str, db: f64) {
        self.gain_elements.insert(name.to_string(), db);
        if let Ok(guard) = self.live_device.lock() {
            if let Some(dev) = guard.as_ref() {
                let _ = dev.set_gain_element(Rx, 0, name.as_bytes(), db);
            }
        }
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

    /// Startet Empfänger-Thread (SoapySDR → Ring Buffer) und DSP-Thread (FFT + Demod → Audio).
    pub fn start(&mut self) -> bool {
        if self.state.lock().unwrap().is_running {
            return false; // bereits aktiv
        }

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
        // Per-Element-Gains überschreiben falls gesetzt
        for (name, &db) in &self.gain_elements {
            let _ = device.set_gain_element(name, db);
        }

        let rx_stream = match device.rx_stream() {
            Ok(s) => s,
            Err(e) => { eprintln!("RX-Stream öffnen fehlgeschlagen: {}", e); return false; }
        };

        // Gerät im shared Arc speichern für Live-Tuning
        {
            let mut ld = self.live_device.lock().unwrap();
            *ld = Some(device.into_inner());
        }

        let rb = HeapRb::<Complex<f32>>::new(SAMPLE_RATE as usize); // 1s Buffer
        let (mut iq_producer_rx, mut iq_consumer) = rb.split();

        let state = Arc::clone(&self.state);
        let demod_mode = self.demod_mode;

        // is_running VOR Spawn setzen damit der Thread beim ersten Check true liest
        {
            let mut state_guard = self.state.lock().unwrap();
            state_guard.is_running = true;
        }

        let state_rx = Arc::clone(&self.state);
        let live_device_rx = Arc::clone(&self.live_device);

        // Empfänger-Thread: SoapySDR → Ring Buffer
        thread::spawn(move || {
            let mut rx_stream = rx_stream;
            if let Err(e) = rx_stream.activate(None) {
                eprintln!("Stream aktivieren fehlgeschlagen: {}", e);
                return;
            }
            let mut buf = vec![Complex::default(); 8192];
            loop {
                {
                    if !state_rx.lock().unwrap().is_running { break; }
                }
                match rx_stream.read(&mut [&mut buf], 100_000) {
                    Ok(len) => {
                        for &s in &buf[..len] {
                            let _ = iq_producer_rx.try_push(s);
                        }
                    }
                    Err(e) => eprintln!("Read-Fehler: {}", e),
                }
            }
            let _ = rx_stream.deactivate(None);
            // Gerät freigeben wenn Thread endet
            let mut ld = live_device_rx.lock().unwrap();
            *ld = None;
        });

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
            let mut audio_buf: Vec<f32> = Vec::with_capacity(FFT_SIZE);

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

                demod.process_into(&buf, &mut audio_buf);
                audio.push_samples(&audio_buf);
            }
        });

        true
    }

    pub fn stop(&mut self) {
        let mut state = self.state.lock().unwrap();
        state.is_running = false;
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
