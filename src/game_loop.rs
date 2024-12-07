use vello::kurbo::Point;
use winit::dpi::Position;
use crate::logic_hook::{GameLogic, InputEvent, TickResult};
use crate::logic_hook::TickResult::{Draw, Exit};

pub struct GameLoop {
    pos: Point,
    vel: Point,
}

impl GameLogic for GameLoop {
    fn tick(&mut self, tick: u64, events: impl Iterator<Item=InputEvent>) -> TickResult {



        for event in events {
            match event {
                InputEvent::ExitRequested => return Exit,
                InputEvent::Touch(touch) => {
                }
                _ => {}
            }
        }

        let y = tick % 90;
        let y = y as f64 * 10.0;
        Draw(Point::new(200.0, 100.0 + y))
    }
}