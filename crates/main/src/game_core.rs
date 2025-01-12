use egui::{hex_color, emath, lerp, pos2, remap, vec2, Color32, Frame, Pos2, Rect, Ui, WidgetText};
use egui_dock::TabViewer;
use epaint::{PathStroke};
use crate::event_handling::EguiGuiExtendContext;
use crate::logic_hook::{GameContext, GameLoop, SynchronousLoop};



pub struct GameCore;

impl GameCore {
    pub fn new() -> (Gui, LogicLoop) {
        (Gui::new(), LogicLoop::new())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Tab {
    Main,
    Drawn,
    Logs,
}

pub struct Gui {
    tree: egui_dock::DockState<Tab>,
}

impl Gui {
    pub fn new() -> Self {
        Self {
            tree: egui_dock::DockState::new(vec![Tab::Main, Tab::Drawn, Tab::Logs]),
        }
    }
}

fn draw(ui: &mut Ui) {
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

struct Viewer<'a> {
    ctx: &'a EguiGuiExtendContext,
}

impl<'a> TabViewer for Viewer<'a> {
    type Tab = Tab;

    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
        match tab {
            Tab::Main => "Main".into(),
            Tab::Drawn => "Drawn".into(),
            Tab::Logs => "Logs".into(),
        }
    }

    fn ui(&mut self, ui: &mut Ui, tab: &mut Tab) {
        match tab {
            Tab::Main => {
                ui.text_edit_singleline(&mut "other test".to_owned());
                    if ui.button("nothing button").clicked() {
                }
            }
            Tab::Drawn => {
                draw(ui);
            }
            Tab::Logs => {
                egui::ScrollArea::horizontal().show(ui, |ui| ui.add(self.ctx.log_widget()));
            }
        }
    }
}

impl SynchronousLoop for Gui {
    fn update_gui(&mut self, ctx: &mut EguiGuiExtendContext) {
        egui_dock::DockArea::new(&mut self.tree)
            .style(egui_dock::Style::from_egui(ctx.style().as_ref()))
            .show(ctx, &mut Viewer{ ctx });
    }
}

pub struct LogicLoop {

}

impl LogicLoop {
    pub fn new() -> Self {
        Self {}
    }
}

impl GameLoop for LogicLoop {
    fn tick(&mut self, _ctx: &GameContext) {

    }
}