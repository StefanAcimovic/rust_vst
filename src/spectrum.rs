use std::sync::{Arc, Mutex};
use rustfft::{FftPlanner, num_complex::Complex};

pub struct SpectrumAnalyzer {
    fft_size: usize,
    planner: FftPlanner<f32>,
    window: Vec<f32>,
    spectrum: Arc<Mutex<Vec<f32>>>,
    smoothed_spectrum: Vec<f32>,
    sample_buffer: Vec<f32>,
    buffer_pos: usize,
    sample_rate: f32,
    smoothing_factor: f32,
}

impl SpectrumAnalyzer {
    pub fn new(fft_size: usize, sample_rate: f32) -> (Self, Arc<Mutex<Vec<f32>>>) {
        let planner = FftPlanner::new();
        
        // Hann window for better frequency resolution
        let window = Self::create_hann_window(fft_size);
        
        let spectrum = Arc::new(Mutex::new(vec![0.0; fft_size / 2]));
        let spectrum_clone = spectrum.clone();
        
        let analyzer = Self {
            fft_size,
            planner,
            window,
            spectrum,
            smoothed_spectrum: vec![0.0f32; fft_size / 2],
            sample_buffer: vec![0.0; fft_size],
            buffer_pos: 0,
            sample_rate,
            smoothing_factor: 0.15,  // Lower = smoother (15% new, 85% old)
        };
        
        (analyzer, spectrum_clone)
    }
    
    fn create_hann_window(size: usize) -> Vec<f32> {
        (0..size)
            .map(|n| {
                let n = n as f32;
                let size = size as f32;
                0.5 * (1.0 - ((2.0 * std::f32::consts::PI * n) / (size - 1.0)).cos())
            })
            .collect()
    }
    
    pub fn process_sample(&mut self, sample: f32) {
        self.sample_buffer[self.buffer_pos] = sample * self.window[self.buffer_pos];
        self.buffer_pos += 1;
        
        if self.buffer_pos >= self.fft_size {
            self.compute_fft();
            self.buffer_pos = 0;
        }
    }
    
    fn compute_fft(&mut self) {
        let mut input: Vec<Complex<f32>> = self.sample_buffer
            .iter()
            .map(|&s| Complex { re: s, im: 0.0 })
            .collect();
        
        let fft = self.planner.plan_fft_forward(self.fft_size);
        fft.process(&mut input);
        
        // Compute magnitude spectrum with proper normalization and smoothing
        let spectrum: Vec<f32> = input
            .iter()
            .take(self.fft_size / 2)
            .enumerate()
            .map(|(i, c)| {
                let magnitude = (c.re * c.re + c.im * c.im).sqrt();
                let normalized = (magnitude / (self.fft_size as f32)) * 2.0;
                let db = 20.0 * (normalized + 1e-10).log10();
                let clamped = db.clamp(-100.0, 0.0);
                clamped
            })
            .collect();
        
        // Apply exponential smoothing to each bin
        for i in 0..spectrum.len() {
            self.smoothed_spectrum[i] = 
                self.smoothed_spectrum[i] * (1.0 - self.smoothing_factor) +
                spectrum[i] * self.smoothing_factor;
        }
        
        if let Ok(mut spec) = self.spectrum.lock() {
            spec.copy_from_slice(&self.smoothed_spectrum);
        }
    }
    
    pub fn get_freq_at_bin(&self, bin: usize) -> f32 {
        (bin as f32 * self.sample_rate) / (self.fft_size as f32)
    }
}
