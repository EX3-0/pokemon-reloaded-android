//! Prints the built-in touch-control layout as JSON.
//!
//! The launcher app draws its editor preview from this, so the button positions come from the same
//! table the engine uses instead of a hand-kept copy. Regenerate it with `tools/generate-layout.sh`.

use gm8emulator::touch_controls::{self, Dir, Label, Shape};

fn center(shape: Shape) -> (f32, f32) {
    match shape {
        Shape::Circle { cx, cy, .. } | Shape::Rect { cx, cy, .. } => (cx, cy),
    }
}

fn label_json(label: Label) -> String {
    match label {
        Label::None => String::new(),
        Label::Arrow(dir) => {
            match dir {
                Dir::Up => "UP",
                Dir::Down => "DOWN",
                Dir::Left => "LEFT",
                Dir::Right => "RIGHT",
            }
            .to_string()
        },
        Label::Text(text) => text.to_string(),
    }
}

fn main() {
    let mut out = String::from("{\n  \"controls\": [\n");
    for (index, control) in touch_controls::CONTROLS.iter().enumerate() {
        let (kind, size) = match control.shape {
            Shape::Circle { r, .. } => ("circle", format!("\"r\": {:.4}", r)),
            Shape::Rect { w, h, .. } => ("rect", format!("\"w\": {:.4}, \"h\": {:.4}", w, h)),
        };
        let (cx, cy) = center(control.shape);
        let comma = if index + 1 == touch_controls::CONTROLS.len() { "" } else { "," };
        out.push_str(&format!(
            "    {{ \"name\": \"{}\", \"kind\": \"{}\", \"cx\": {:.4}, \"cy\": {:.4}, {}, \"label\": \"{}\", \"contexts\": {} }}{}\n",
            control.name,
            kind,
            cx,
            cy,
            size,
            label_json(control.label),
            control.contexts,
            comma
        ));
    }
    out.push_str("  ],\n");
    out.push_str(&format!(
        "  \"joystick\": {{ \"name\": \"joystick\", \"cx\": {:.4}, \"cy\": {:.4}, \"r\": {:.4} }}\n",
        touch_controls::JOYSTICK_CX,
        touch_controls::JOYSTICK_CY,
        touch_controls::JOYSTICK_R
    ));
    out.push_str("}\n");
    print!("{}", out);
}
