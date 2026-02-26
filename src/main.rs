mod audio;
mod audio_engine;
mod ui;
mod spectrum;
mod spectrum_ui;

use eframe::egui;
use std::sync::{Arc, Mutex};
use audio::DistortionParams;
use audio_engine::AudioEngine;
use spectrum::SpectrumAnalyzer;

struct DistortionPedalApp {
    params: Arc<Mutex<DistortionParams>>,
    _audio_engine: Option<AudioEngine>,
    spectrum_analyzer: Option<SpectrumAnalyzer>,
    spectrum_data: Option<Arc<Mutex<Vec<f32>>>>,
    show_spectrum: bool,
    sample_rate: f32,
}

impl Default for DistortionPedalApp {
    fn default() -> Self {
        let params = Arc::new(Mutex::new(DistortionParams::default()));
        
        let (audio_engine, spectrum_data) = match AudioEngine::new(params.clone()) {
            Ok(engine) => {
                let spec_data = engine.spectrum_data.clone();
                (Some(engine), Some(spec_data))
            },
            Err(e) => {
                eprintln!("Failed to start audio engine: {}", e);
                (None, None)
            }
        };
        
        Self {
            params,
            _audio_engine: audio_engine,
            spectrum_analyzer: None,
            spectrum_data,
            show_spectrum: false,
            sample_rate: 44100.0,
        }
    }
}

impl eframe::App for DistortionPedalApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Request continuous repaint for smooth UI
        ctx.request_repaint();
        
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(egui::Color32::from_rgb(20, 20, 25)))
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(20.0);
                    
                    // Title
                    ui.heading(
                        egui::RichText::new("🎸 RUST TEST DISTORZIJA")
                            .size(42.0)
                            .color(egui::Color32::from_rgb(100, 200, 255))
                    );
                    
                    ui.add_space(15.0);
                    
                    ui.label(
                        egui::RichText::new("Stefan Acimovic")
                            .size(14.0)
                            .color(egui::Color32::from_rgb(150, 150, 160))
                    );
                    
                    ui.add_space(30.0);
                    
                    // Knobs
                    ui.horizontal(|ui| {
                        ui.add_space(40.0);
                        
                        let mut params = self.params.lock().unwrap();
                        
                        // Drive knob
                        ui::knob_widget(ui, &mut params.drive, "DRIVE", 100.0);
                        ui.add_space(30.0);
                        
                        // Tone knob
                        ui::knob_widget(ui, &mut params.tone, "TONE", 100.0);
                        ui.add_space(30.0);
                        
                        // Level knob
                        ui::knob_widget(ui, &mut params.level, "LEVEL", 100.0);
                    });
                    
                    ui.add_space(40.0);
                    
                    // Spectrum analyzer toggle
                    ui.horizontal_top(|ui| {
                        ui.add_space(20.0);
                        let button = ui.button(
                            egui::RichText::new(if self.show_spectrum { "▼ Sakrij Spektar" } else { "▶ Pokazi Spektar" })
                                .size(12.0)
                        );
                        if button.clicked() {
                            self.show_spectrum = !self.show_spectrum;
                            eprintln!("[MAIN] Spectrum toggle: {}", self.show_spectrum);
                        }
                    });
                    
                    ui.add_space(15.0);
                    
                    // Spectrum analyzer panel
                    if self.show_spectrum {
                        if let Some(spectrum_data) = &self.spectrum_data {
                            spectrum_ui::spectrum_panel(ui, spectrum_data, self.sample_rate);
                        }
                    }
                    
                    ui.add_space(20.0);
                    
                    // Status indicator
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("●")
                                .size(16.0)
                                .color(egui::Color32::from_rgb(50, 255, 50))
                        );
                        ui.label(
                            egui::RichText::new("ACTIVE")
                                .size(13.0)
                                .color(egui::Color32::from_rgb(150, 150, 160))
                        );
                    });
                    
                    ui.add_space(10.0);
                    
                    ui.label(
                        egui::RichText::new("Input: Scarlett 4i4 - Channel 1")
                            .size(11.0)
                            .color(egui::Color32::from_rgb(100, 100, 110))
                    );
                });
            });
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([700.0, 800.0])
            .with_resizable(true),
        ..Default::default()
    };
    
    eframe::run_native(
        "Distortion Pedal",
        options,
        Box::new(|_cc| Ok(Box::new(DistortionPedalApp::default()))),
    )
}
