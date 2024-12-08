use std::fs;
use log::error;
use rdev::{listen, EventType};
use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
pub struct Offset {
    pub x: i32,
    pub y: i32
}

fn main() {
    let offset = fs::read_to_string("offset.toml").map_or(Offset::default(), |file| toml::from_str(&file).unwrap());
    let macro_defs = fs::read_to_string("macros.txt")
        .expect("Couldn't read 'macros.txt'");
    let mut macros = match mackerel::parse_file(&macro_defs) {
        Ok((_, macros)) => macros,
        Err(err) => {
            error!("Couldn't parse macros.txt\n{:?}", &err);
            return;
        },
    };
    for mc in macros.iter_mut() {
        mc.apply_offset(offset.x, offset.y);
    }
    let mut mouse_pos = (0, 0);
    if let Err(err) = listen(move |event| {
        let event_type = match event.event_type {
            EventType::MouseMove { x, y } => {
                mouse_pos = (x as i32 + offset.x, y as i32 + offset.y);
                EventType::MouseMove { x: x + offset.x as f64, y: y + offset.y as f64 }
            },
            x => x
        };
        for script in &macros {
            script.on_event(&event_type, mouse_pos);
        }
    }) {
        error!("Couldn't listen to events\n{:?}", &err);
    }
}