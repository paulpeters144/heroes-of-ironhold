use crate::GameFont;
use macroquad::prelude::*;

pub(super) fn wrap_text(font: &GameFont, text: &str, max_width: f32) -> String {
    if max_width <= 0.0 {
        return text.to_string();
    }

    let mut lines: Vec<String> = Vec::new();
    for paragraph in text.split('\n') {
        let mut current = String::new();
        for word in paragraph.split_whitespace() {
            let candidate = if current.is_empty() {
                word.to_string()
            } else {
                format!("{current} {word}")
            };
            let dims = measure_text(&candidate, Some(&font.font), font.size, 1.0);
            if dims.width > max_width && !current.is_empty() {
                lines.push(std::mem::take(&mut current));
                current = word.to_string();
            } else {
                current = candidate;
            }
        }
        if !current.is_empty() {
            lines.push(current);
        }
    }

    lines.join("\n")
}
