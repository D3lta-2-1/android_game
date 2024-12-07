use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};
use vello::kurbo::Point;
use winit::event::Touch;
use crate::event_handler::LogicHandler;

pub enum InputEvent {
    ExitRequested,
    Touch(Touch),
}

pub struct DrawContent {
    pub pos: Point,
    pub tick: u64,
}

pub enum TickResult {
    Draw(Point),
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
    pub fn new(logic: impl GameLogic + 'static, tick_length: Duration) -> (LogicHook, Receiver<DrawContent>) {
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
    draw_sender: Sender<DrawContent>,
    tick_length: Duration,
}

impl<T: GameLogic> GameClock<T> {
    fn main_loop(&mut self) {
        let mut tick_count = 0u64;
        let mut next_frame = Instant::now();
        loop {
            let result = self.logic.tick(tick_count, self.input_receiver.try_iter());
            next_frame += self.tick_length;

            match result {
                TickResult::Draw(pos) => self.draw_sender.send(DrawContent {
                    pos,
                    tick: tick_count,
                }).unwrap(),
                TickResult::Exit => break,
            }
            let duration = next_frame.duration_since(Instant::now());
            thread::sleep(duration);
            tick_count += 1;
        }
    }
}

