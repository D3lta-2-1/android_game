use egui::WidgetText;
use egui_dock::TabViewer;
use crate::logic_hook::{GameContext, GameLoop, SynchronousLoop};

pub struct GameCore;

impl GameCore {
    pub fn new() -> (Gui, LogicLoop) {
        (Gui::new(), LogicLoop::new())
    }
}


pub struct Gui {
    tree: egui_dock::DockState<String>,
}

impl Gui {
    pub fn new() -> Self {
        Self {
            tree: egui_dock::DockState::new(vec!["README.md".to_owned(), "CHANGELOG.md".to_owned()]),
        }
    }
}

struct Viewer;

impl TabViewer for Viewer {
    type Tab = String;

    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
        tab.as_str().into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, _title: &mut String) {
        ui.label("Welcome on my newly rust-made android app!");
        ui.label("Here a bunch of garbage !");
    }
}

impl SynchronousLoop for Gui {
    fn update_gui(&mut self, ctx: &egui::Context, toasts: &mut egui_notify::Toasts) {
        egui::TopBottomPanel::bottom("buttons").show(ctx, |ui| {
            ui.text_edit_singleline(&mut "other test".to_owned());
            if ui.button("nothing button").clicked() {

            }
            if ui.button("toasts").clicked() {
                toasts.info("Hello, people!".to_owned());
            }
        });
        egui_dock::DockArea::new(&mut self.tree)
            .style(egui_dock::Style::from_egui(ctx.style().as_ref()))
            .show(ctx, &mut Viewer{});
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