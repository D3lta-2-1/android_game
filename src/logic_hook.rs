use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};
use winit::event::Touch;
use crate::event_handling::LogicHandler;
use crate::rendering::drawing::CommandBundle;

pub enum InputEvent {
    ExitRequested,
    Touch(Touch),
}

pub enum TickResult {
    Draw/*(Vec<DrawCommand>)*/,
    Exit,
}

pub trait GameLogic: Send {
    fn tick(&mut self, tick: u64, events: impl Iterator<Item = InputEvent>) -> TickResult;
}

pub struct LogicHook {
    input_sender: Sender<InputEvent>,
    game_thread: Option<thread::JoinHandle<()>>,
}

impl LogicHook {
    pub fn new(logic: impl GameLogic + 'static, tick_length: Duration) -> (LogicHook, Receiver<CommandBundle>) {
        let (input_sender, input_receiver) = mpsc::channel();
        let (draw_sender, draw_receiver) = mpsc::channel();

        let mut clock = GameClock { logic, input_receiver, draw_sender, tick_length };
        let game_thread = Some(thread::spawn(
            move || clock.main_loop()
        ));

        let hook = LogicHook { input_sender, game_thread };
        (hook, draw_receiver)
    }
}

impl LogicHandler for LogicHook {
    fn exit(&mut self) {
        self.input_sender.send(InputEvent::ExitRequested).unwrap();
        self.game_thread.take().unwrap().join().unwrap();
    }

    fn touch_event(&mut self, touch: Touch) {
        self.input_sender.send(InputEvent::Touch(touch)).unwrap();
    }
}

struct GameClock<T: GameLogic> {
    logic: T,
    input_receiver: Receiver<InputEvent>,
    draw_sender: Sender<CommandBundle>,
    tick_length: Duration,
}

impl<T: GameLogic> GameClock<T> {
    fn main_loop(&mut self) {
        let mut tick_count = 0u64;
        let mut next_tick = Instant::now();
        loop {
            let result = self.logic.tick(tick_count, self.input_receiver.try_iter());
            next_tick += self.tick_length;

            let bundle = match result {
                TickResult::Draw/*(commands)*/ => CommandBundle::new_empty() /*{
                    commands,
                    // TODO: add camera transform
                    camera_transform: Interpolation::None(Transform::from_pos((0.0, 0.0))),
                    tick_start: next_tick,
                }*/,
                TickResult::Exit => break,
            };
            let duration = next_tick.duration_since(Instant::now());
            thread::sleep(duration);
            //sent the Bundle 1 tick behind, else we might get interpolation issues
            self.draw_sender.send(bundle).unwrap();
            tick_count += 1;
        }
    }
}

