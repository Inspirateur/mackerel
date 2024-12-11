use std::{thread, time::Duration};
use rdev::{simulate, EventType};
use log::{debug, error};

#[derive(Debug)]
pub(crate) enum Trigger {
    Single(EventType),
    Combo(Vec<EventType>),
}

impl Trigger {
    pub fn on_event(&self, event: &EventType) -> bool {
        match self {
            Self::Single(trigger) => event == trigger,
            // TODO: implement input combo such as CTRL + left click 
            // this will probably require another layer of abstraction between raw events and triggers,
            // to gather inputs that are sent together in a single struct
            Self::Combo(_) => false
        }
    }
}

#[derive(Debug)]
pub struct Macro {
    pub(crate) trigger: Trigger,
    pub(crate) actions: Vec<Action>
}

impl Macro {
    pub fn on_event(&self, event: &EventType, mouse_pos: (i32, i32)) {
        if !self.trigger.on_event(event) {
            return;
        }
        debug!(">>> Macro triggered");
        for action in self.actions.iter() {
            debug!("{action:?}");
            let event_type = match action {
                Action::Event(event_type) => *event_type,
                Action::MoveToStart => EventType::MouseMove { x: mouse_pos.0 as f64, y: mouse_pos.1 as f64 },
                Action::Wait(ms) => { 
                    thread::sleep(Duration::from_millis(*ms as u64));
                    continue;
                }
            };
            if let Err(_) = simulate(&event_type) {
                error!("Couldn't simulate {:?}", &event_type);
            };
            thread::sleep(Duration::from_millis(40));
        }
        debug!("<<< done")
    }

    pub fn apply_offset(&mut self, offset_x: i32, offset_y: i32) {
        for action in &mut self.actions {
            match action {
                Action::Event(EventType::MouseMove { x, y }) => {
                    *x = *x + offset_x as f64;
                    *y = *y + offset_y as f64;
                },
                _ => {},
            }
        }
    } 
}

#[derive(Debug)]
pub(crate) enum Action {
    Event(EventType),
    MoveToStart,
    Wait(u32)
}