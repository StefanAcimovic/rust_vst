use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct DistortionParams {
    pub drive: f32,   // 0.0 to 1.0
    pub tone: f32,    // 0.0 to 1.0
    pub level: f32,   // 0.0 to 1.0
}

impl Default for DistortionParams {
    fn default() -> Self {
        Self {
            drive: 0.5,
            tone: 0.5,
            level: 0.7,
        }
    }
}

pub struct AudioProcessor {
    params: Arc<Mutex<DistortionParams>>,
    sample_rate: f32,
    // Simple one-pole lowpass filter state for tone control
    filter_state: f32,
}

impl AudioProcessor {
    pub fn new(params: Arc<Mutex<DistortionParams>>, sample_rate: f32) -> Self {
        Self {
            params,
            sample_rate,
            filter_state: 0.0,
        }
    }

    pub fn process_sample(&mut self, input: f32) -> f32 {
        let params = self.params.lock().unwrap();
        
        let gain = 1.0 + params.drive * params.drive * 300.0;
        let driven = input * gain;
        
        let distorted = driven.tanh();
        
        let asymmetric = distorted + (distorted * distorted * distorted * 0.15);
        
        let cutoff_freq = 300.0 + params.tone * 4700.0;
        let rc = 1.0 / (cutoff_freq * 2.0 * std::f32::consts::PI);
        let dt = 1.0 / self.sample_rate;
        let alpha = dt / (rc + dt);
        
        self.filter_state = self.filter_state + alpha * (asymmetric - self.filter_state);
        
        self.filter_state * params.level * 1.2
    }
}
