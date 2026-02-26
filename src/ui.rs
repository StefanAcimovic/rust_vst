use eframe::egui;
use egui::{Color32, FontId, Pos2, Sense, Stroke, Vec2};

pub fn knob_widget(
    ui: &mut egui::Ui,
    value: &mut f32,
    label: &str,
    size: f32,
) -> egui::Response {
    let (rect, mut response) = ui.allocate_exact_size(
        Vec2::new(size, size + 30.0),
        Sense::click_and_drag(),
    );
    
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let center = Pos2::new(rect.center().x, rect.top() + size / 2.0);
        let radius = size / 2.0 - 6.0;
        
        // Handle interaction
        if response.dragged() {
            let delta = response.drag_delta().y;
            *value = (*value - delta * 0.005).clamp(0.0, 1.0);
            response.mark_changed();
        }
        
        // Background circle (track)
        painter.circle(
            center,
            radius,
            Color32::from_rgb(30, 30, 35),
            Stroke::new(3.0, Color32::from_rgb(60, 60, 70)),
        );
        
        // Value arc
        let start_angle = -2.5; // Start angle in radians
        let end_angle = 0.7; // End angle in radians
        let value_angle = start_angle + *value * (end_angle - start_angle);
        
        // Draw value arc
        let steps = 40;
        for i in 0..steps {
            let t = i as f32 / steps as f32;
            let angle = start_angle + t * (*value * (end_angle - start_angle));
            let next_t = (i + 1) as f32 / steps as f32;
            let next_angle = start_angle + next_t * (*value * (end_angle - start_angle));
            
            let p1 = Pos2::new(
                center.x + radius * angle.cos(),
                center.y + radius * angle.sin(),
            );
            let p2 = Pos2::new(
                center.x + radius * next_angle.cos(),
                center.y + radius * next_angle.sin(),
            );
            
            painter.line_segment(
                [p1, p2],
                Stroke::new(4.0, Color32::from_rgb(100, 200, 255)),
            );
        }
        
        // Indicator line
        let indicator_start = radius * 0.3;
        let indicator_end = radius * 0.85;
        let indicator_pos1 = Pos2::new(
            center.x + indicator_start * value_angle.cos(),
            center.y + indicator_start * value_angle.sin(),
        );
        let indicator_pos2 = Pos2::new(
            center.x + indicator_end * value_angle.cos(),
            center.y + indicator_end * value_angle.sin(),
        );
        
        painter.line_segment(
            [indicator_pos1, indicator_pos2],
            Stroke::new(3.0, Color32::from_rgb(255, 255, 255)),
        );
        
        // Center dot
        painter.circle_filled(center, 5.0, Color32::from_rgb(50, 50, 60));
        
        // Label
        let label_pos = Pos2::new(center.x, rect.bottom() - 15.0);
        painter.text(
            label_pos,
            egui::Align2::CENTER_CENTER,
            label,
            FontId::proportional(16.0),
            Color32::from_rgb(200, 200, 210),
        );
        
        // Value text
        let value_text = format!("{:.0}%", *value * 100.0);
        let value_pos = Pos2::new(center.x, rect.bottom() - 2.0);
        painter.text(
            value_pos,
            egui::Align2::CENTER_CENTER,
            value_text,
            FontId::proportional(12.0),
            Color32::from_rgb(140, 140, 150),
        );
    }
    
    response
}
