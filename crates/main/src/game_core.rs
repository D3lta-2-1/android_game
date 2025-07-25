use crate::logic_hook::{GameContext, GameLoop, SynchronousLoop};
use crate::world::constraints::ConstraintWidget;
use crate::world::{GameContent, Solver, WorldSnapshot};
use egui::{Color32, Frame, Pos2, Shape, Stroke, Ui, WidgetText, vec2};
use egui_dock::{NodeIndex, TabViewer};
use egui_plot::{Legend, Line, Plot, PlotPoint};
use nalgebra::Vector2;
use running_context::event_handling::EguiGuiExtendContext;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;

pub struct GameCore;

impl GameCore {
    pub fn new(time_step: Duration) -> (Gui, LogicLoop) {
        let (graphic_sender, graphic_receiver) = std::sync::mpsc::channel();
        let (event_sender, event_receiver) = std::sync::mpsc::channel();
        (
            Gui::new(graphic_receiver, event_sender),
            LogicLoop::new(graphic_sender, event_receiver, time_step),
        )
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Tab {
    Button,
    World,
    Plots,
    Stats,
}

struct Event {
    simulation: SimulationContent,
    solver: Solver,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum SimulationContent {
    Simple,
    Double,
    Triple,
    Rope,
    Rail,
    Structure,
    Pulley,
    PulleyAndRail,
    Bridge,
    BridgeSoft,
}

impl SimulationContent {
    const LIST: [SimulationContent; 10] = [
        SimulationContent::Simple,
        SimulationContent::Double,
        SimulationContent::Triple,
        SimulationContent::Rope,
        SimulationContent::Rail,
        SimulationContent::Structure,
        SimulationContent::Pulley,
        SimulationContent::PulleyAndRail,
        SimulationContent::Bridge,
        SimulationContent::BridgeSoft,
    ];
}

pub struct Gui {
    graphic_receiver: Receiver<WorldSnapshot>,
    dock_viewer: DockViewer,
    tree: egui_dock::DockState<Tab>,
}

impl Gui {

    fn default_view() -> egui_dock::DockState<Tab> {
        let mut tree = egui_dock::DockState::new(vec![Tab::World]);
        let main_surface = tree.main_surface_mut();
        let [_, a] = main_surface.split_right(NodeIndex::root(), 0.5, vec![Tab::Button]);
        main_surface.split_right(a, 0.5, vec![Tab::Stats]);
        main_surface.split_below(a, 0.5, vec![Tab::Plots]);
        tree
    }

    fn new(receiver: Receiver<WorldSnapshot>, event_sender: Sender<Event>) -> Self {
        Self {
            graphic_receiver: receiver,
            dock_viewer: DockViewer {
                snapshot: WorldSnapshot::default(),
                sender: event_sender,
                kinetic_energy: vec![],
                potential_energy: vec![],
                elastic_energy: vec![],
                mechanical_energy: vec![],
                precision_factor: vec![],
                selected_simulation: SimulationContent::Double,
                selected_solver: Solver::HybridV3,
                should_clear_graph: false,
            },
            tree: Self::default_view() //egui_dock::DockState::new(vec![Tab::World, Tab::Button, Tab::Plots, Tab::Stats]).,
        }
    }
}

struct DockViewer {
    snapshot: WorldSnapshot,
    sender: Sender<Event>,
    kinetic_energy: Vec<PlotPoint>,
    potential_energy: Vec<PlotPoint>,
    elastic_energy: Vec<PlotPoint>,
    mechanical_energy: Vec<PlotPoint>,
    precision_factor: Vec<PlotPoint>,
    selected_simulation: SimulationContent,
    selected_solver: Solver,
    should_clear_graph: bool,
}

impl TabViewer for DockViewer {
    type Tab = Tab;

    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
        match tab {
            Tab::Button => "Main".into(),
            Tab::World => "Pendulum".into(),
            Tab::Plots => "Plots".into(),
            Tab::Stats => "Stats".into(),
        }
    }

    fn ui(&mut self, ui: &mut Ui, tab: &mut Tab) {
        match tab {
            Tab::Button => self.display_button(ui),
            Tab::World => self.draw_simulation(ui),
            Tab::Plots => self.draw_plot(ui),
            Tab::Stats => self.display_stats(ui),
        }
    }
}

impl DockViewer {
    fn display_button(&mut self, ui: &mut Ui) {
        let mut send_event = false;
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label("Simulation");
                for simulation_content in SimulationContent::LIST.into_iter() {
                    if ui
                        .selectable_value(
                            &mut self.selected_simulation,
                            simulation_content,
                            format!("{:?}", simulation_content),
                        )
                        .clicked()
                    {
                        send_event = true;
                    }
                }
            });
            ui.vertical(|ui| {
                ui.label("Solver");
                for solver in Solver::LIST.into_iter() {
                    if ui
                        .selectable_value(
                            &mut self.selected_solver,
                            solver,
                            format!("{:?}", solver),
                        )
                        .clicked()
                    {
                        send_event = true;
                    }
                }
            });
        });
        if send_event {
            self.sender
                .send(Event {
                    simulation: self.selected_simulation,
                    solver: self.selected_solver,
                })
                .unwrap();
        }
        ui.label("- soft simulations are only soft if the solver supports it, otherwise they are rigid");
        ui.label("- precision factor is the number of zero after the decimal point in the mean violation of the constraints, it doesn't have any mean if the simulation have soft parts. It's a good indicator of the precision of the simulation, the higher the better.");
        ui.label("- the mechanical energy is the sum of the kinetic, potential and elastic energy, it should be constant in a perfect simulation.");
    }

    fn draw_simulation(&self, ui: &mut Ui) {
        Frame::canvas(ui.style()).show(ui, |ui| {
            let desired_size = vec2(ui.available_width(), ui.available_height());
            let (_id, rect) = ui.allocate_space(desired_size);

            let center = rect.center();

            let to_screen_coordinates = |p: Vector2<f32>| {
                let mut p = Pos2::new(p.x * 70.0, p.y * -70.0);
                p += center.to_vec2();
                p
            };

            let mut shapes = vec![];

            for (widgets, force) in self.snapshot.links.iter() {
                let lerp_color = if *force >= 0.0 {
                    Color32::WHITE.lerp_to_gamma(Color32::LIGHT_BLUE, *force * 0.008)
                } else {
                    Color32::WHITE.lerp_to_gamma(Color32::LIGHT_RED, -*force * 0.008)
                };

                match widgets {
                    ConstraintWidget::Link(a, b) => {
                        let pos_a = self.snapshot.pos[*a];
                        let pos_b = self.snapshot.pos[*b];
                        shapes.push(Shape::line_segment(
                            [to_screen_coordinates(pos_a), to_screen_coordinates(pos_b)],
                            Stroke::new(3.0, lerp_color),
                        ));
                    }
                    ConstraintWidget::Anchor(a, anchor) => {
                        let pos = to_screen_coordinates(self.snapshot.pos[*a]);
                        let anchor = to_screen_coordinates(*anchor);

                        shapes.push(Shape::line_segment(
                            [pos, anchor],
                            Stroke::new(3.0, lerp_color),
                        ));
                        shapes.push(Shape::circle_filled(anchor, 5.0, Color32::BLUE))
                    }
                    ConstraintWidget::Plane(normal, d) => {
                        let (u, v) = rect.center().into();
                        let d = d + normal.dot(&Vector2::new(u, -v));
                        let a = normal.x;
                        let b = -normal.y;
                        let x1 = rect.left();
                        let x2 = rect.right();
                        let y1 = rect.top();
                        let y2 = rect.bottom();

                        //ax + by = d
                        // => y = (d - ax) / b
                        // => x = (d - by) / a
                        let points = [
                            Pos2::new(x1, (d - a * x1) / b),
                            Pos2::new(x2, (d - a * x2) / b),
                            Pos2::new((d - b * y1) / a, y1),
                            Pos2::new((d - b * y2) / a, y2),
                        ];

                        let mut iter = points.into_iter().filter(|x| rect.contains(*x));

                        let mut array = || Some([iter.next()?, iter.next()?]);

                        if let Some([a, b]) = array() {
                            shapes.push(Shape::line_segment(
                                [a, b],
                                Stroke::new(3.0, Color32::DARK_GRAY),
                            ));
                        }
                    }
                    ConstraintWidget::Pulley(a, b, anchor_a, anchor_b) => {
                        let pos_a = to_screen_coordinates(self.snapshot.pos[*a]);
                        let pos_b = to_screen_coordinates(self.snapshot.pos[*b]);
                        let anchor_a = to_screen_coordinates(*anchor_a);
                        let anchor_b = to_screen_coordinates(*anchor_b);

                        shapes.push(Shape::line_segment(
                            [pos_a, anchor_a],
                            Stroke::new(3.0, lerp_color),
                        ));
                        shapes.push(Shape::line_segment(
                            [pos_b, anchor_b],
                            Stroke::new(3.0, lerp_color),
                        ));
                        shapes.push(Shape::circle_filled(anchor_a, 5.0, Color32::BLUE));
                        shapes.push(Shape::circle_filled(anchor_b, 5.0, Color32::BLUE));
                    }

                    _ => (),
                }
            }
            shapes.extend(
                self.snapshot.pos.iter().map(|body| {
                    Shape::circle_filled(to_screen_coordinates(*body), 7.0, Color32::RED)
                }),
            );
            ui.painter().extend(shapes);
        });
    }

    fn draw_plot(&mut self, ui: &mut Ui) {
        let mut plot = Plot::new("energy over time").legend(Legend::default());

        if self.should_clear_graph {
            plot = plot.reset();
        }

        plot.show(ui, |plot_ui| {
            let kinetic = Line::new(self.kinetic_energy.as_ref()).name("Kinetic Energy");
            let potential = Line::new(self.potential_energy.as_ref()).name("Potential Energy");
            let mechanical = Line::new(self.mechanical_energy.as_ref()).name("Mechanical Energy");
            plot_ui.line(kinetic);
            plot_ui.line(potential);
            plot_ui.line(mechanical);

            if self.elastic_energy.is_empty() {
                return;
            }

            let elastic = Line::new(self.elastic_energy.as_ref()).name("Elastic Energy");
            plot_ui.line(elastic);
        });
    }

    fn display_stats(&self, ui: &mut Ui) {
        ui.label(format!(
            "time taken to solve: {:?}",
            self.snapshot.calculation_time
        ));

        let mut plot = Plot::new("precision over time").legend(Legend::default());

        if self.should_clear_graph {
            plot = plot.reset();
        }
        plot.show(ui, |plot_ui| {
            let precision = Line::new(self.precision_factor.as_ref()).name("Mean Precision (number of zero after the decimal point)");
            plot_ui.line(precision);
        });
    }
}

impl SynchronousLoop for Gui {
    fn update_gui(&mut self, ctx: &mut EguiGuiExtendContext) {
        self.dock_viewer.should_clear_graph = false;
        if self.dock_viewer.kinetic_energy.len() > 8000 {
            self.dock_viewer.kinetic_energy.clear();
            self.dock_viewer.potential_energy.clear();
            self.dock_viewer.elastic_energy.clear();
            self.dock_viewer.mechanical_energy.clear();
            self.dock_viewer.precision_factor.clear();
        }

        for latest in self.graphic_receiver.try_iter() {
            self.dock_viewer.snapshot = latest;
            let time = self.dock_viewer.snapshot.date as f64;
            let kinetic_energy = self.dock_viewer.snapshot.kinetic_energy as f64;
            let potential_energy = self.dock_viewer.snapshot.potential_energy as f64;
            let elastic_energy = self.dock_viewer.snapshot.elastic_energy as f64;
            let precision_factor = f32::min(-self.dock_viewer.snapshot.violation_mean.log10(), 7.0) as f64;
            if time < self.dock_viewer.kinetic_energy.last().map_or(0.0, |p| p.x) {
                self.dock_viewer.kinetic_energy.clear();
                self.dock_viewer.potential_energy.clear();
                self.dock_viewer.elastic_energy.clear();
                self.dock_viewer.mechanical_energy.clear();
                self.dock_viewer.precision_factor.clear();
                self.dock_viewer.should_clear_graph = true;
            }
            self.dock_viewer
                .kinetic_energy
                .push(PlotPoint::new(time, kinetic_energy));
            self.dock_viewer
                .potential_energy
                .push(PlotPoint::new(time, potential_energy));
            if !self.dock_viewer.elastic_energy.is_empty() || elastic_energy != 0.0 {
                self.dock_viewer
                    .elastic_energy
                    .push(PlotPoint::new(time, elastic_energy));
            }
            self.dock_viewer
                .mechanical_energy
                .push(PlotPoint::new(time, kinetic_energy + potential_energy + elastic_energy));
            self.dock_viewer
                .precision_factor
                .push(PlotPoint::new(time, precision_factor));
        }

        egui_dock::DockArea::new(&mut self.tree)
            .style(egui_dock::Style::from_egui(ctx.style().as_ref()))
            .show(ctx, &mut self.dock_viewer);
    }
}

pub struct LogicLoop {
    simulation: GameContent,
    graphic_sender: Sender<WorldSnapshot>,
    event_receiver: Receiver<Event>,
}

impl LogicLoop {
    fn new(
        graphic_sender: Sender<WorldSnapshot>,
        event_receiver: Receiver<Event>,
        tick_step: Duration,
    ) -> Self {
        let mut simulation = GameContent::empty(tick_step.as_secs_f32());
        simulation.double();
        Self {
            simulation,
            graphic_sender,
            event_receiver,
        }
    }
}

impl GameLoop for LogicLoop {
    fn tick(&mut self, _ctx: &GameContext) {
        if let Ok(Event { simulation, solver }) = self.event_receiver.try_recv() {
            self.simulation.solver = solver;
            match simulation {
                SimulationContent::Simple => self.simulation.simple(),
                SimulationContent::Double => self.simulation.double(),
                SimulationContent::Triple => self.simulation.triple(),
                SimulationContent::Rope => self.simulation.rope(),
                SimulationContent::Rail => self.simulation.rail(),
                SimulationContent::Structure => self.simulation.structure(),
                SimulationContent::Pulley => self.simulation.pulley(),
                SimulationContent::PulleyAndRail => self.simulation.pulley_and_rail(),
                SimulationContent::Bridge => self.simulation.bridge(),
                SimulationContent::BridgeSoft => self.simulation.bridge_soft(),
            };
        }

        self.simulation.solve();
        let snapshot = self.simulation.take_snapshot();
        self.graphic_sender.send(snapshot).unwrap();
    }
}
