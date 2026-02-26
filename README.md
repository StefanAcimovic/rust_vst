# 🎸 Rust Test Distorzija

A real-time guitar distortion pedal with frequency spectrum analyzer, built in Rust.

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)

## Features

- **Real-time audio processing** with ultra-low latency (~6ms CPAL buffer + ~93ms ring buffer)
- **Analog-style distortion** with tanh waveshaping and harmonic asymmetry
- **Tone control** with variable lowpass filtering (300Hz - 5kHz)
- **Live FFT spectrum analyzer** with exponential smoothing
- **Modern GUI** with interactive circular knobs (egui framework)
- **Logarithmic frequency display** (50Hz - 20kHz) for better low-frequency resolution

## Algorithms

### 🔥 Drive (Distortion)

**File:** `src/audio.rs:37-42`

```rust
// Exponential gain curve
let gain = 1.0 + params.drive * params.drive * 300.0;  // 1x to 301x
let driven = input * gain;

// Tanh waveshaping for smooth saturation
let distorted = driven.tanh();

// Asymmetry for tube-like warmth (adds 3rd harmonic)
let asymmetric = distorted + (distorted * distorted * distorted * 0.15);
```

**How it works:**
- Drive knob controls gain with exponential curve (drive² × 300)
- Tanh function provides smooth, musical saturation
- Cubic term adds asymmetry for analog tube-like character

### 🎛️ Tone (Lowpass Filter)

**File:** `src/audio.rs:47-54`

```rust
// One-pole lowpass filter (RC filter simulation)
let cutoff_freq = 300.0 + params.tone * 4700.0;  // 300Hz to 5kHz
let rc = 1.0 / (cutoff_freq * 2.0 * PI);
let dt = 1.0 / sample_rate;
let alpha = dt / (rc + dt);

// Apply filter
filter_state = filter_state + alpha * (input - filter_state);
```

**How it works:**
- Simple one-pole RC lowpass filter
- Tone knob: 0.0 = dark (300Hz cutoff), 1.0 = bright (5kHz cutoff)
- Processes the distorted signal for tone shaping

### 🔊 Level (Output Gain)

**File:** `src/audio.rs:57`

```rust
output = filter_state * params.level * 1.2;  // 1.2x makeup gain
```

**How it works:**
- Linear volume control
- 1.2× makeup gain compensates for filtering losses

### 📊 Frequency Spectrum Analyzer

**FFT Processing:** `src/spectrum.rs:60-89`

```rust
// 1. Apply Hann window to reduce spectral leakage
sample_buffer[i] = sample * hann_window[i];

// 2. Compute 2048-point FFT
fft.process(&mut input);

// 3. Calculate magnitude for each bin
magnitude = sqrt(real² + imag²);

// 4. Normalize and convert to dB
normalized = (magnitude / fft_size) * 2.0;
db = 20.0 * log10(normalized + 1e-10);

// 5. Exponential smoothing (85% old, 15% new)
smoothed[i] = smoothed[i] * 0.85 + db * 0.15;
```

**Display Algorithm:** `src/spectrum_ui.rs:38-77`

```rust
// Logarithmic frequency scale (50Hz to 20kHz)
freq = exp(log(50) + x_ratio * (log(20000) - log(50)));

// Average FFT bins for each display bar (128 bars)
avg_magnitude = average(spectrum[start_bin..end_bin]);

// Color gradient based on magnitude
if magnitude < -80dB: blue (quiet)
if magnitude < -60dB: cyan (medium)
if magnitude > -60dB: yellow/orange (loud)
```

**How it works:**
- 2048-point FFT with Hann windowing for frequency analysis
- Logarithmic scale gives more resolution to bass frequencies (like human hearing)
- Exponential smoothing prevents jittery animation
- 128 display bars, each averaging multiple FFT bins
- Range: 50Hz-20kHz (audible spectrum only)

## Requirements

- **Rust** 1.70 or later
- **Windows** (uses WASAPI via CPAL)
- **Audio Interface**: Focusrite Scarlett or similar (automatically uses default devices)

## Installation & Running

```bash
# Clone the repository
git clone https://github.com/StefanAcimovic/rust_vst.git
cd rust_vst

# Build and run
cargo run --release
```

**Note:** Use `--release` flag for optimal audio performance (debug builds may have dropouts).

## Usage

1. **Connect your guitar** to your audio interface (Input 1)
2. **Run the application**
3. **Adjust knobs:**
   - **Drive**: Controls distortion amount (0% = clean, 100% = heavy distortion)
   - **Tone**: Adjusts brightness (0% = dark/warm, 100% = bright/cutting)
   - **Level**: Output volume
4. **Toggle spectrum analyzer** with the "Show Spectrum" button

## Technical Details

### Audio Configuration
- **Sample Rate**: 44.1kHz
- **CPAL Buffer**: 256 samples (~5.8ms)
- **Ring Buffer**: 4096 samples (~92.9ms)
- **Total Latency**: ~32-40ms (acceptable for real-time guitar playing)
- **Channels**: Mono input (channel 0), stereo output

### Signal Chain

```
Input → Drive Gain → Tanh Waveshaping → Asymmetry → Tone Filter → Level → Output
                                                                            ↓
                                                                    FFT Analyzer → GUI
```

### Dependencies

- **eframe/egui**: Modern immediate-mode GUI framework
- **CPAL**: Cross-platform audio library (low-latency audio I/O)
- **rustfft**: Fast Fourier Transform implementation
- **ringbuf**: Lock-free ring buffer for audio samples

## Project Structure

```
src/
├── main.rs           # GUI application and main loop
├── audio.rs          # Distortion algorithms (Drive, Tone, Level)
├── audio_engine.rs   # Real-time audio I/O with CPAL
├── spectrum.rs       # FFT computation and smoothing
├── spectrum_ui.rs    # Frequency spectrum visualization
└── ui.rs             # Circular knob widget
```

## License

MIT License - feel free to use this code for learning or your own projects!

## Author

Stefan Acimovic

---

*Built with ❤️ and Rust*
