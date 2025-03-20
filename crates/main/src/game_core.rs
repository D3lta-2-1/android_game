use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;
use egui::{hex_color, emath, lerp, pos2, remap, vec2, Color32, Frame, Pos2, Rect, Ui, WidgetText};
use egui_dock::TabViewer;
use epaint::{PathStroke, Stroke};
use nalgebra::Vector2;
use running_context::event_handling::EguiGuiExtendContext;
use crate::logic_hook::{GameContext, GameLoop, SynchronousLoop};
use crate::pendulum::PendulumSystem;

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
    Triple
}

pub struct Gui {
    graphic_receiver: Receiver<Vec<Vector2<f32>>>,
    event_sender: Sender<Event>,
    last_pos: Vec<Vector2<f32>>,
    tree: egui_dock::DockState<Tab>,
}

impl Gui {
    pub fn new(receiver: Receiver<Vec<Vector2<f32>>>, event_sender: Sender<Event>) -> Self {
        Self {
            graphic_receiver: receiver,
            event_sender,
            last_pos: vec![],
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

fn draw_pendule(ui: &mut Ui, positions: &[Vector2<f32>]) {
    Frame::canvas(ui.style()).show(ui, |ui| {
        let desired_size = vec2(ui.available_width(), ui.available_height());
        let (_id, rect) = ui.allocate_space(desired_size);

        let center= rect.center();


        let mut shapes = vec![];

        let iter = std::iter::once(Vector2::new(0.0, 0.0)).chain(positions.iter().copied());
        let mut points: Vec<Pos2> = iter.map(|position| {
            pos2(position.x, position.y)
        }).collect();

        points.iter_mut().for_each(|p| {
            p.x *= 70.0;
            p.y *= -70.0; // because y is inverted on screen
            *p += center.to_vec2()
        });

        shapes.push(epaint::Shape::Circle(epaint::CircleShape{
            center: points[0],
            radius: 70.0,
            fill: Color32::from_rgba_premultiplied(0, 0, 0, 0),
            stroke: Stroke::new(3.0, Color32::DARK_GRAY)
        }));


        shapes.push(epaint::Shape::line(
            points.clone(),
            PathStroke::new(3.0, Color32::WHITE)
        ));
        shapes.push(epaint::Shape::circle_filled(points[0], 5.0, Color32::BLUE));
        for i in 1..points.len() {
            shapes.push(epaint::Shape::circle_filled(points[i], 7.0, Color32::RED));
        }

        ui.painter().extend(shapes);
    });
}

struct Viewer<'a> {
    pos: &'a [Vector2<f32>],
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
                if ui.button("simple pendulum").clicked() {
                    self.sender.send(Event::Simple).unwrap();
                }
                if ui.button("double pendulum").clicked() {
                    self.sender.send(Event::Double).unwrap();
                }
                if ui.button("triple pendulum").clicked() {
                    self.sender.send(Event::Triple).unwrap();
                }
            }
            Tab::Waves => {
                draw_waves(ui);
            }
            Tab::Pendulum => {
                draw_pendule(ui, self.pos);
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
    simulation: PendulumSystem,
    graphic_sender: Sender<Vec<Vector2<f32>>>,
    event_receiver: Receiver<Event>,
}

impl LogicLoop {
    fn new(graphic_sender: Sender<Vec<Vector2<f32>>>, event_receiver: Receiver<Event>, tick_step: Duration) -> Self {
        Self {
            simulation: PendulumSystem::double(tick_step.as_secs_f32()),
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
                Event::Simple => PendulumSystem::simple(time_step),
                Event::Double => PendulumSystem::double(time_step),
                Event::Triple => PendulumSystem::triple(time_step),
            };
        }

        self.simulation.integrate();
        self.simulation.solve();
        self.graphic_sender.send(self.simulation.bodies.iter().map(|b| b.position).collect()).unwrap();
    }
}