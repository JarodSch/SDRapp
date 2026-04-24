use soapysdr::Direction::Rx;
use num_complex::Complex;

#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub label: String,
    pub args: String,
    pub driver: String,
}

pub struct SdrDevice {
    device: soapysdr::Device,
    sample_rate: f64,
}

impl SdrDevice {
    /// Gibt alle angeschlossenen SoapySDR-Geräte zurück.
    /// Gibt leere Liste zurück wenn keine Hardware gefunden.
    pub fn enumerate() -> Vec<DeviceInfo> {
        match soapysdr::enumerate("") {
            Ok(list) => list.iter().map(|args| DeviceInfo {
                label: args.get("label").unwrap_or("Unknown").to_string(),
                args: args.to_string(),
                driver: args.get("driver").unwrap_or("unknown").to_string(),
            }).collect(),
            Err(e) => { eprintln!("SoapySDR enumerate error: {e}"); vec![] }
        }
    }

    /// Öffnet ein SoapySDR-Gerät. Gibt Fehler zurück wenn nicht gefunden.
    pub fn open(args: &str) -> Result<Self, soapysdr::Error> {
        let device = soapysdr::Device::new(args)?;
        Ok(Self { device, sample_rate: 2_048_000.0 })
    }

    /// Konfiguriert Frequenz, Gain und Sample-Rate.
    pub fn configure(
        &mut self,
        frequency_hz: u64,
        gain_db: f64,
        sample_rate: f64,
    ) -> Result<(), soapysdr::Error> {
        self.device.set_sample_rate(Rx, 0, sample_rate)?;
        self.device.set_frequency(Rx, 0, frequency_hz as f64, ())?;
        self.device.set_gain(Rx, 0, gain_db)?;
        self.sample_rate = sample_rate;
        Ok(())
    }

    pub fn sample_rate(&self) -> f64 { self.sample_rate }

    /// Erstellt einen RX-Stream für IQ-Samples.
    pub fn rx_stream(&self) -> Result<soapysdr::RxStream<Complex<f32>>, soapysdr::Error> {
        self.device.rx_stream(&[0])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enumerate_does_not_panic() {
        // Gibt entweder Geräte oder leere Liste — darf nie paniken
        let devices = SdrDevice::enumerate();
        println!("Gefundene Geräte: {}", devices.len());
        for d in &devices {
            assert!(!d.label.is_empty(), "Label darf nicht leer sein");
            assert!(!d.driver.is_empty(), "Driver darf nicht leer sein");
        }
    }

    #[test]
    fn test_open_invalid_args_returns_error() {
        let result = SdrDevice::open("driver=nonexistent_xyz_abc");
        assert!(result.is_err(), "Ungültige Args sollten Err zurückgeben");
    }

    #[test]
    fn test_device_info_fields() {
        // DeviceInfo muss Clone und Debug implementieren
        let info = DeviceInfo {
            label: "Test Device".to_string(),
            args: "driver=test".to_string(),
            driver: "test".to_string(),
        };
        let cloned = info.clone();
        assert_eq!(info.label, cloned.label);
        let debug_str = format!("{:?}", info);
        assert!(debug_str.contains("Test Device"));
    }
}
