use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Stream, StreamConfig};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::audio::{AudioProcessor, DistortionParams};
use crate::spectrum::SpectrumAnalyzer;
use ringbuf::traits::{Consumer, Producer, Split, Observer};

pub struct AudioEngine {
    _input_stream: Stream,
    _output_stream: Stream,
    pub spectrum_data: Arc<Mutex<Vec<f32>>>,
}

impl AudioEngine {
    pub fn new(params: Arc<Mutex<DistortionParams>>) -> Result<Self, Box<dyn std::error::Error>> {
        let host = cpal::default_host();
        
        // Get audio devices
        let input_device = host.default_input_device()
            .ok_or("No input device available")?;
        let output_device = host.default_output_device()
            .ok_or("No output device available")?;
        
        println!("Input device: {}", input_device.name()?);
        println!("Output device: {}", output_device.name()?);
        
        let mut config: StreamConfig = input_device.default_input_config()?.into();
        // Set buffer size to 256 samples for balance between latency and stability (~6ms at 44.1kHz)
        config.buffer_size = cpal::BufferSize::Fixed(256);
        let sample_rate = config.sample_rate.0 as f32;
        
        println!("Sample rate: {} Hz", sample_rate);
        println!("Channels: {}", config.channels);
        println!("Buffer size: 256 samples (~{:.1}ms latency)", 256.0 / sample_rate * 1000.0);
        println!("Ring buffer: 4096 samples (~{:.1}ms)", 4096.0 / sample_rate * 1000.0);
        
        // Create ring buffer - larger to prevent clipping/dropouts
        let ring = ringbuf::HeapRb::<f32>::new(4096);
        let (mut producer, mut consumer) = ring.split();
        
        // Diagnostic counters
        let input_counter = Arc::new(AtomicUsize::new(0));
        let output_counter = Arc::new(AtomicUsize::new(0));
        let buffer_fill = Arc::new(AtomicUsize::new(0));
        
        let input_counter_clone = input_counter.clone();
        let output_counter_clone = output_counter.clone();
        let buffer_fill_clone = buffer_fill.clone();
        
        // Input stream - reads from Scarlett input
        let input_stream = input_device.build_input_stream(
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                let mut pushed = 0;
                // Take only channel 0 (Input 1 on Scarlett)
                for frame in data.chunks(config.channels as usize) {
                    if let Some(&sample) = frame.get(0) {
                        if producer.try_push(sample).is_ok() {
                            pushed += 1;
                        }
                    }
                }
                input_counter_clone.fetch_add(pushed, Ordering::Relaxed);
            },
            |err| eprintln!("Input stream error: {}", err),
            None,
        )?;
        
        // Output stream - writes to audio output
        let mut processor = AudioProcessor::new(params, sample_rate);
        let (mut spectrum, spectrum_data) = SpectrumAnalyzer::new(2048, sample_rate);
        
        let output_stream = output_device.build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                let mut popped = 0;
                for frame in data.chunks_mut(config.channels as usize) {
                    // Get input sample from ring buffer
                    let input_sample = consumer.try_pop().unwrap_or(0.0);
                    popped += 1;
                    
                    // Process the sample
                    let output_sample = processor.process_sample(input_sample);
                    
                    // Feed to spectrum analyzer
                    spectrum.process_sample(output_sample);
                    
                    // Write to all output channels
                    for sample in frame.iter_mut() {
                        *sample = output_sample;
                    }
                }
                output_counter_clone.fetch_add(popped, Ordering::Relaxed);
                buffer_fill_clone.store(consumer.occupied_len(), Ordering::Relaxed);
            },
            |err| eprintln!("Output stream error: {}", err),
            None,
        )?;
        
        input_stream.play()?;
        output_stream.play()?;
        
        println!("Audio engine started!");
        println!("Ring buffer capacity: {} samples ({:.1}ms)", 2048, 2048.0 / sample_rate * 1000.0);
        
        // Spawn a thread to monitor buffer status
        let sample_rate_monitor = sample_rate;
        std::thread::spawn(move || {
            let mut last_input = 0;
            let mut last_output = 0;
            loop {
                std::thread::sleep(std::time::Duration::from_secs(2));
                let input = input_counter.load(Ordering::Relaxed);
                let output = output_counter.load(Ordering::Relaxed);
                let fill = buffer_fill.load(Ordering::Relaxed);
                
                let input_rate = (input - last_input) / 2;
                let output_rate = (output - last_output) / 2;
                
                println!("\n=== Audio Stats ===");
                println!("Input rate: {} samples/s ({:.1}x expected)", input_rate, input_rate as f32 / sample_rate_monitor);
                println!("Output rate: {} samples/s ({:.1}x expected)", output_rate, output_rate as f32 / sample_rate_monitor);
                println!("Buffer fill: {}/2048 samples ({:.1}ms of audio)", fill, fill as f32 / sample_rate_monitor * 1000.0);
                println!("Estimated latency: {:.1}ms", (256.0 + fill as f32) / sample_rate_monitor * 1000.0);
                
                last_input = input;
                last_output = output;
            }
        });
        
        Ok(Self {
            _input_stream: input_stream,
            _output_stream: output_stream,
            spectrum_data,
        })
    }
}
