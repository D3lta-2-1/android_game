use std::mem::replace;
use std::sync::Arc;
use image::load_from_memory;
use vello::kurbo::{Affine, Circle, Ellipse, Line, RoundedRect, Stroke, Vec2};
use vello::peniko::{Blob, Color, Format, Image};
use vello::Scene;
use winit::event::{Touch, TouchPhase};
use crate::logic_hook::{GameLogic, InputEvent, TickResult};
use crate::logic_hook::TickResult::{Draw, Exit};
use crate::rendering::drawing::{DrawCommand, Interpolation, Transform};

fn add_shapes_to_scene(scene: &mut Scene) {
    // Draw an outlined rectangle
    let stroke = Stroke::new(6.0);
    let rect = RoundedRect::new(10.0, 10.0, 240.0, 240.0, 20.0);
    let rect_stroke_color = Color::rgb(0.9804, 0.702, 0.5294);
    scene.stroke(&stroke, Affine::IDENTITY, rect_stroke_color, None, &rect);

    // Draw a filled circle
    let circle = Circle::new((420.0, 200.0), 120.0);
    let circle_fill_color = Color::rgb(0.9529, 0.5451, 0.6588);
    scene.fill(
        vello::peniko::Fill::NonZero,
        Affine::IDENTITY,
        circle_fill_color,
        None,
        &circle,
    );

    // Draw a filled ellipse
    let ellipse = Ellipse::new((250.0, 420.0), (100.0, 160.0), -90.0);
    let ellipse_fill_color = Color::rgb(0.7961, 0.651, 0.9686);
    scene.fill(
        vello::peniko::Fill::NonZero,
        Affine::IDENTITY,
        ellipse_fill_color,
        None,
        &ellipse,
    );

    // Draw a straight line
    let line = Line::new((260.0, 20.0), (620.0, 100.0));
    let line_stroke_color = Color::rgb(0.5373, 0.7059, 0.9804);
    scene.stroke(&stroke, Affine::IDENTITY, line_stroke_color, None, &line);
}


pub struct GameLoop {
    scene: Arc<Scene>,
    last_touch_pos: Option<Vec2>,
    pos: Vec2,
    vel: Vec2,
    _image: Image
}

impl GameLoop {
    pub fn new() -> Self {
        let mut scene = Scene::new();

        let image = load_from_memory(include_bytes!("saucisse.jpg")).unwrap();
        let rgba = image.to_rgba8();
        let width = rgba.width();
        let height = rgba.height();


        let blob = Blob::new(Arc::new(rgba.into_raw()));

        let image = Image::new(blob, Format::Rgba8, width, height);
        add_shapes_to_scene(&mut scene);
        //scene.draw_image(&image, Affine::scale(16.0));


        Self {
            scene: Arc::new(scene),
            last_touch_pos: None,
            pos: Vec2::new(200.0, 100.0),
            vel: Vec2::ZERO,
            _image: image
        }
    }
}

impl GameLogic for GameLoop {
    fn tick(&mut self, _tick: u64, events: impl Iterator<Item=InputEvent>) -> TickResult {

        let mut touch_translation = Vec2::new(0.0, 0.0);
        for event in events {
            match event {
                InputEvent::ExitRequested => return Exit,
                InputEvent::Touch(Touch{ phase, location, .. }) => {
                    if phase == TouchPhase::Ended || phase == TouchPhase::Cancelled {
                        self.last_touch_pos.take();
                    }
                    else {
                        let pos = Vec2::new(location.x, location.y);
                        if let Some(last_pos) = self.last_touch_pos.replace(pos) {
                            touch_translation += pos - last_pos;
                        }
                    }
                },
                _ => {}
            }
        }
        if touch_translation == Vec2::ZERO {
            self.vel *= 0.95;
        } else {
            self.vel = touch_translation;
        }

        let new_pos = self.pos + self.vel;
        let command = DrawCommand::new(
            self.scene.clone(),
            Interpolation::Linear(
                Transform::from_pos(replace(&mut self.pos, new_pos)),
                Transform::from_pos(self.pos),
            )
        );
        Draw(vec![command])
    }
}