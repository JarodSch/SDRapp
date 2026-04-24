use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::{traits::*, HeapProd, HeapRb};

pub struct AudioOutput {
    _stream: cpal::Stream, // Stream muss am Leben bleiben
    pub producer: HeapProd<f32>,
}

impl AudioOutput {
    pub fn new(sample_rate: u32) -> Result<Self, Box<dyn std::error::Error>> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or("Kein Audio-Ausgabegerät gefunden")?;

        let config = cpal::StreamConfig {
            channels: 1,
            sample_rate: cpal::SampleRate(sample_rate),
            buffer_size: cpal::BufferSize::Default,
        };

        let rb = HeapRb::<f32>::new(sample_rate as usize); // 1 Sekunde Buffer
        let (producer, mut consumer) = rb.split();

        let stream = device.build_output_stream(
            &config,
            move |data: &mut [f32], _| {
                for sample in data.iter_mut() {
                    *sample = consumer.try_pop().unwrap_or(0.0);
                }
            },
            |err| eprintln!("Audio-Fehler: {}", err),
            None,
        )?;

        stream.play()?;
        Ok(Self {
            _stream: stream,
            producer,
        })
    }

    pub fn push_samples(&mut self, samples: &[f32]) {
        for &s in samples {
            // Blockiert nicht — voller Buffer wird ignoriert (Drop)
            let _ = self.producer.try_push(s);
        }
    }
}
