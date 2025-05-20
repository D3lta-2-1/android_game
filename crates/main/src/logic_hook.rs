use running_context::event_handling::{EguiGuiExtendContext, LogicHandler};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

/**
 *   ``LogicHook`` is a struct that is used to run the game logic in a separate thread.
 *   It decomposes game logic and GUI logic into separate threads, allowing the GUI to run at a different rate than the game logic.
 *   Tick rate is controlled by the tick_length parameter.
 *   If a tick is longer than the tick_length, the game will slow down. In an attempt to recover ``LogicHook`` will not wait between nexts ticks, and tick as fast as possible.
 *   To synchronize the game logic and frame rendering, LogicHook extensively use mpsc channels.
 **/
pub struct LogicHook<T: SynchronousLoop> {
    sync_loop: T,
    game_thread: Option<thread::JoinHandle<()>>,
    keep_running: Arc<AtomicBool>,
}

impl<T: SynchronousLoop> LogicHook<T> {
    // this is kinda ugly... I might look into dynamic dispatching
    pub fn new(
        (sync_loop, mut logic): (T, impl GameLoop + 'static),
        tick_length: Duration,
    ) -> Self {
        let keep_running = Arc::new(AtomicBool::new(true));

        let mut game_context = GameContext::new_empty(tick_length, keep_running.clone());
        let game_thread = Some(thread::spawn(move || {
            // Logic loop
            game_context.start();
            while game_context.wait_until_next_tick() {
                logic.tick(&game_context);
            }
            logic.exit();
        }));

        let hook = LogicHook {
            sync_loop,
            game_thread,
            keep_running,
        };
        hook
    }
}

impl<T: SynchronousLoop> LogicHandler for LogicHook<T> {
    fn update_gui(&mut self, ctx: &mut EguiGuiExtendContext) {
        self.sync_loop.update_gui(ctx);
    }

    fn exit(&mut self) {
        self.keep_running.store(false, Ordering::SeqCst);
        self.sync_loop.exit();
        self.game_thread.take().unwrap().join().unwrap(); // a thread do not survive the end of the program, so we must wait for it to finish
    }
}

pub struct GameContext {
    next_tick: Instant,
    tick_length: Duration,
    tick_count: u64,
    keep_running: Arc<AtomicBool>,
}
impl GameContext {
    fn new_empty(tick_length: Duration, keep_running: Arc<AtomicBool>) -> Self {
        Self {
            next_tick: Instant::now(),
            tick_length,
            tick_count: 0,
            keep_running,
        }
    }

    fn start(&mut self) {
        self.next_tick = Instant::now();
    }

    fn wait_until_next_tick(&mut self) -> bool {
        if self.keep_running.load(Ordering::Acquire) {
            self.next_tick += self.tick_length;
            let duration = self.next_tick.duration_since(Instant::now());
            thread::sleep(duration);
            self.tick_count += 1;
            true
        } else {
            false
        }
    }
}

pub trait GameLoop: Send {
    fn tick(&mut self, ctx: &GameContext);
    fn exit(&mut self) {}
}

pub trait SynchronousLoop {
    fn update_gui(&mut self, ctx: &mut EguiGuiExtendContext);
    //TODO: forward inputs
    fn exit(&mut self) {}
}
