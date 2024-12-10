use std::fs;
use log::{error, LevelFilter};
use rdev::{listen, EventType};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Offset {
    pub x: i32,
    pub y: i32,
    pub s: f64,
}

impl Default for Offset {
    fn default() -> Self {
        Self {
            x: 0, y: 0, s: 1.
        }
    }
}

fn main() {
    env_logger::builder().filter_level(LevelFilter::Debug).init();
    let offset = fs::read_to_string("offset.toml").map_or(Offset::default(), |file| toml::from_str(&file).unwrap_or_else(|err| {
        error!("Error in 'offset.toml': {err}\nUsing default values instead.");
        Offset::default()
    }));
    let Ok(macro_defs) = fs::read_to_string("macros.txt") else {
        error!("You don't have any macros :(\nCreate a 'macros.txt' file and write macros with our scripting language!\nSee https://github.com/Inspirateur/mackerel for help");
        return;
    };
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
                mouse_pos = ((x*offset.s) as i32 + offset.x, (y*offset.s) as i32 + offset.y);
                EventType::MouseMove { x: x*offset.s + offset.x as f64, y: y*offset.s + offset.y as f64 }
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