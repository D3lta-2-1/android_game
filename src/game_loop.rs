use crate::logic_hook::{GameLogic, InputEvent, TickResult};

pub struct GameLoop {

}

impl GameLoop {
    pub fn new() -> Self {
        Self{}
    }
}

impl GameLogic for GameLoop {
    fn tick(&mut self, _tick: u64, events: impl Iterator<Item=InputEvent>) -> TickResult {
        for event in events {
            match event {
                InputEvent::ExitRequested => return TickResult::Exit,
                _ => (),
            }
        }

        TickResult::Draw/*(vec![])*/
    }
}