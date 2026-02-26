use eframe::egui;
use egui::{Color32, FontId, Pos2, Stroke, Vec2};
use std::sync::{Arc, Mutex};

pub fn spectrum_panel(
    ui: &mut egui::Ui,
    spectrum_data: &Arc<Mutex<Vec<f32>>>,
    sample_rate: f32,
) {
    egui::Frame::default()
        .fill(Color32::from_rgb(25, 25, 30))
        .stroke(Stroke::new(2.0, Color32::from_rgb(80, 80, 90)))
        .inner_margin(10.0)
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new("📊 FREQ SPEKTAR")
                    .size(14.0)
                    .color(Color32::from_rgb(100, 200, 255))
            );
            
            ui.add_space(5.0);
            
            let rect = ui.available_rect_before_wrap();
            let height = 180.0;
            ui.allocate_space(Vec2::new(ui.available_width(), height));
            
            let mut rect = rect;
            rect.set_height(height);
            
            if rect.width() > 0.0 && rect.height() > 0.0 {
                let painter = ui.painter();
                
                // Background
                painter.rect_filled(rect, 2.0, Color32::from_rgb(20, 20, 25));
                painter.rect_stroke(rect, 2.0, Stroke::new(1.0, Color32::from_rgb(60, 60, 70)));
                
                // Get spectrum data with logarithmic frequency scale
                if let Ok(spectrum) = spectrum_data.lock() {
                    let bin_width_hz = sample_rate / (spectrum.len() as f32 * 2.0);
                    let start_freq: f32 = 50.0;
                    let end_freq: f32 = 20000.0;
                    
                    // Logarithmic scale
                    let log_start = start_freq.ln();
                    let log_end = end_freq.ln();
                    let log_range = log_end - log_start;
                    
                    let num_display_bins = 128;  // Display 128 bars for detailed frequency view
                    let bin_width = rect.width() / num_display_bins as f32;
                    
                    // Draw bars for each frequency region
                    for display_idx in 0..num_display_bins {
                        let x_ratio = display_idx as f32 / num_display_bins as f32;
                        let x_ratio_next = (display_idx + 1) as f32 / num_display_bins as f32;
                        
                        // Convert screen position to frequency (logarithmic)
                        let freq_at_pos = (log_start + log_range * x_ratio).exp();
                        let freq_at_next = (log_start + log_range * x_ratio_next).exp();
                        
                        // Convert frequencies to bin indices
                        let bin_at_pos = (freq_at_pos / bin_width_hz) as usize;
                        let bin_at_next = (freq_at_next / bin_width_hz) as usize;
                        
                        let start_bin = bin_at_pos.max(0).min(spectrum.len());
                        let end_bin = bin_at_next.max(0).min(spectrum.len());
                        
                        if start_bin >= end_bin {
                            continue;
                        }
                        
                        // Average magnitudes in this region for smooth envelope
                        let avg_magnitude = spectrum[start_bin..end_bin].iter().sum::<f32>() / (end_bin - start_bin) as f32;
                        
                        // Normalize magnitude
                        let norm = ((avg_magnitude + 100.0) / 100.0).clamp(0.0, 1.0);
                        let bar_height = norm * (rect.height() - 30.0);
                        
                        let x = rect.left() + (display_idx as f32) * bin_width;
                        let y_top = rect.bottom() - bar_height;
                        
                        // Color gradient
                        let color = if norm < 0.2 {
                            Color32::from_rgb(40, 80, 120)
                        } else if norm < 0.4 {
                            Color32::from_rgb(50, 120, 180)
                        } else if norm < 0.6 {
                            Color32::from_rgb(100, 200, 255)
                        } else if norm < 0.8 {
                            Color32::from_rgb(200, 220, 100)
                        } else {
                            Color32::from_rgb(255, 150, 100)
                        };
                        
                        // Draw bar
                        painter.rect_filled(
                            egui::Rect::from_two_pos(
                                Pos2::new(x, y_top),
                                Pos2::new(x + bin_width - 2.0, rect.bottom() - 25.0)
                            ),
                            0.5,
                            color
                        );
                    }
                    
                    // Draw frequency labels (50Hz to 20kHz)
                    let label_freqs: Vec<f32> = vec![50.0, 100.0, 200.0, 500.0, 1000.0, 2000.0, 5000.0, 10000.0, 20000.0];
                    for freq in label_freqs {
                        if freq >= start_freq && freq <= end_freq {
                            let log_freq = freq.ln();
                            let x_ratio = (log_freq - log_start) / log_range;
                            let x = rect.left() + x_ratio * rect.width();
                            
                            let label = if freq >= 1000.0 {
                                format!("{:.0}k", freq / 1000.0)
                            } else {
                                format!("{:.0}", freq)
                            };
                            painter.text(
                                Pos2::new(x, rect.bottom() - 5.0),
                                egui::Align2::CENTER_CENTER,
                                label,
                                FontId::proportional(9.0),
                                Color32::from_rgb(120, 120, 130)
                            );
                        }
                    }
                } else {
                    eprintln!("[UI] Failed to acquire spectrum lock!");
                }
            }
        });
}
