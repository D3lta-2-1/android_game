use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;
use egui::{hex_color, emath, lerp, pos2, remap, vec2, Color32, Frame, Pos2, Rect, Ui, WidgetText};
use egui_dock::TabViewer;
use epaint::PathStroke;
use nalgebra::Vector2;
use running_context::event_handling::EguiGuiExtendContext;
use crate::logic_hook::{GameContext, GameLoop, SynchronousLoop};
use crate::world::{World, WorldSnapshot};
use crate::world::constraints::ConstraintWidget;

pub struct GameCore;

impl GameCore {
    pub fn new(time_step: Duration) -> (Gui, LogicLoop) {
        let (graphic_sender, graphic_receiver) = std::sync::mpsc::channel();
        let (event_sender, event_receiver) = std::sync::mpsc::channel();
        (Gui::new(graphic_receiver, event_sender), LogicLoop::new(graphic_sender, event_receiver, time_step))
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Tab {
    Main,
    Waves,
    Pendulum,
}

enum Event {
    Simple,
    Double,
    Triple,
    Rope,
    Rail,
    Square
}

pub struct Gui {
    graphic_receiver: Receiver<WorldSnapshot>,
    event_sender: Sender<Event>,
    last_pos: WorldSnapshot,
    tree: egui_dock::DockState<Tab>,
}

impl Gui {
    fn new(receiver: Receiver<WorldSnapshot>, event_sender: Sender<Event>) -> Self {
        Self {
            graphic_receiver: receiver,
            event_sender,
            last_pos: WorldSnapshot::default(),
            tree: egui_dock::DockState::new(vec![Tab::Pendulum, Tab::Main, Tab::Waves]),
        }
    }
}

fn draw_waves(ui: &mut Ui) {
    Frame::canvas(ui.style()).show(ui, |ui| {
        ui.ctx().request_repaint();
        let time = ui.input(|i| i.time);

        let desired_size = vec2(ui.available_width(), ui.available_height()); //* vec2(1.0, 0.35);
        let (_id, rect) = ui.allocate_space(desired_size);

        let to_screen =
            emath::RectTransform::from_to(Rect::from_x_y_ranges(0.0..=1.0, -1.0..=1.0), rect);

        let mut shapes = vec![];

        for &mode in &[2, 3, 5] {
            let mode = mode as f64;
            let n = 120;
            let speed = 1.5;

            let points: Vec<Pos2> = (0..=n)
                .map(|i| {
                    let t = i as f64 / (n as f64);
                    let amp = (time * speed * mode).sin() / mode;
                    let y = amp * (t * std::f64::consts::TAU / 2.0 * mode).sin();
                    to_screen * pos2(t as f32, y as f32)
                })
                .collect();

            let thickness = 10.0 / mode as f32;
            shapes.push(epaint::Shape::line(
                points,
                PathStroke::new_uv(thickness, move |rect, p| {
                    let t = remap(p.x, rect.x_range(), -1.0..=1.0).abs();
                    let center_color = hex_color!("#5BCEFA");
                    let outer_color = hex_color!("#F5A9B8");

                    Color32::from_rgb(
                        lerp(center_color.r() as f32..=outer_color.r() as f32, t) as u8,
                        lerp(center_color.g() as f32..=outer_color.g() as f32, t) as u8,
                        lerp(center_color.b() as f32..=outer_color.b() as f32, t) as u8,
                    )
                })

            ));
        }
        ui.painter().extend(shapes);
    });
}

fn draw_simulation(ui: &mut Ui, snapshot: &WorldSnapshot) {
    Frame::canvas(ui.style()).show(ui, |ui| {
        let desired_size = vec2(ui.available_width(), ui.available_height());
        let (_id, rect) = ui.allocate_space(desired_size);

        let center= rect.center();

        let to_screen_coordinates = |p : Vector2<f32>| {
            let mut p = Pos2::new(p.x * 70.0, p.y * -70.0);
            p += center.to_vec2();
            p
        };

        let mut shapes = vec![];

        for (widgets, force) in snapshot.links.iter() {

            let lerp_color = if *force >= 0.0 {
                Color32::WHITE.lerp_to_gamma(Color32::LIGHT_BLUE, *force)
            } else {
                Color32::WHITE.lerp_to_gamma(Color32::LIGHT_RED, -*force)
            };


            match widgets {
                ConstraintWidget::Link(a, b) => {
                    let pos_a = snapshot.pos[*a];
                    let pos_b = snapshot.pos[*b];
                    shapes.push(epaint::Shape::line_segment(
                        [to_screen_coordinates(pos_a), to_screen_coordinates(pos_b)],
                        PathStroke::new(3.0, lerp_color)
                    ));
                },
                ConstraintWidget::Anchor(a, anchor) => {
                    let pos = to_screen_coordinates(snapshot.pos[*a]);
                    let anchor = to_screen_coordinates(*anchor);

                    shapes.push(epaint::Shape::line_segment(
                        [pos, anchor],
                        PathStroke::new(3.0, lerp_color)
                    ));
                    shapes.push(epaint::Shape::circle_filled(
                        anchor,
                        5.0,
                        Color32::BLUE
                    ))
                },
                ConstraintWidget::Horizontal(y) => {

                    let y = y * -70.0 + center.y;
                    shapes.push(epaint::Shape::line_segment(
                        [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
                        PathStroke::new(3.0, Color32::DARK_GRAY)
                    ));
                },
                _ => (),
            }
        }
        shapes.extend(snapshot.pos.iter().map(|body|
            epaint::Shape::circle_filled(
                to_screen_coordinates(*body),
                7.0,
                Color32::RED
            )
        ));
        ui.painter().extend(shapes);
    });
}

struct Viewer<'a> {
    pos: &'a WorldSnapshot,
    sender: &'a Sender<Event>,
}

impl<'a> TabViewer for Viewer<'a> {
    type Tab = Tab;

    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
        match tab {
            Tab::Main => "Main".into(),
            Tab::Waves => "Drawn".into(),
            Tab::Pendulum => "Pendulum".into(),
        }
    }

    fn ui(&mut self, ui: &mut Ui, tab: &mut Tab) {
        match tab {
            Tab::Main => {
                if ui.button("simple world").clicked() {
                    self.sender.send(Event::Simple).unwrap();
                }
                if ui.button("double world").clicked() {
                    self.sender.send(Event::Double).unwrap();
                }
                if ui.button("triple world").clicked() {
                    self.sender.send(Event::Triple).unwrap();
                }
                if ui.button("rope").clicked() {
                    self.sender.send(Event::Rope).unwrap();
                }
                if ui.button("rail").clicked() {
                    self.sender.send(Event::Rail).unwrap();
                }
                if ui.button("square").clicked() {
                    self.sender.send(Event::Square).unwrap()
                }
            }
            Tab::Waves => {
                draw_waves(ui);
            }
            Tab::Pendulum => {
                draw_simulation(ui, self.pos);
            }
        }
    }
}

impl SynchronousLoop for Gui {
    fn update_gui(&mut self, ctx: &mut EguiGuiExtendContext) {

        for pos in self.graphic_receiver.try_iter() {
            self.last_pos = pos;
        }

        egui_dock::DockArea::new(&mut self.tree)
            .style(egui_dock::Style::from_egui(ctx.style().as_ref()))
            .show(ctx, &mut Viewer{
                pos: &self.last_pos,
                sender: &self.event_sender
            });
    }
}

pub struct LogicLoop {
    simulation: World,
    graphic_sender: Sender<WorldSnapshot>,
    event_receiver: Receiver<Event>,
}

impl LogicLoop {
    fn new(graphic_sender: Sender<WorldSnapshot>, event_receiver: Receiver<Event>, tick_step: Duration) -> Self {
        Self {
            simulation: World::double(tick_step.as_secs_f32()),
            graphic_sender,
            event_receiver,
        }
    }
}

impl GameLoop for LogicLoop {
    fn tick(&mut self, _ctx: &GameContext) {

        if let Ok(event) = self.event_receiver.try_recv() {
            let time_step = self.simulation.time_step;
            self.simulation = match event {
                Event::Simple => World::simple(time_step),
                Event::Double => World::double(time_step),
                Event::Triple => World::triple(time_step),
                Event::Rope => World::rope(time_step),
                Event::Rail => World::pendulum_in_rail(time_step),
                Event::Square => World::square(time_step),
            };
        }

        self.simulation.integrate();
        let snapshot = self.simulation.solve();
        self.graphic_sender.send(snapshot).unwrap();
    }
}