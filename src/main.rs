use quicksilver::{
    geom::{Line, Rectangle, Circle, Shape, Transform, Vector, Scalar},
    graphics::{
        Background::{Col, Img},
        Background,
        Color, Font, FontStyle, Drawable, Mesh
    },
    input::Key,
    lifecycle::{run, Asset, Settings, State, Window},
    load_file, Future, Result,
};
use serde::{Deserialize, Serialize};
use std::str::from_utf8;

#[derive(Serialize, Deserialize)]
struct MapMessage {
    points: Vec<Vector>,
}

struct Map {
    lines: Vec<Line>,
}

impl MapMessage {
    fn extract_map(&mut self) -> Map {
        let mut lines: Vec<Line> = Vec::new();
        let mut last_point = self.points[0];
        for point in self.points.iter().skip(1) {
            lines.push(Line::new(last_point.clone(), point.clone()));
            last_point = point.clone();
        }
        Map{ lines }
    }
}

impl Drawable for Map {
    fn draw<'a>(&self, mesh: &mut Mesh, bkg: Background<'a>, trans: Transform, z: impl Scalar) {
        for line in self.lines.iter() {
                line.draw(mesh, bkg, trans, z);
        }
    }
}

struct LunarModule {
    velocity: Vector,
    position: Vector,
    attitude: f32,
}

impl LunarModule {
    fn new(position: Vector) -> LunarModule {
        LunarModule {
            velocity: Vector::new(0.0, 0.0),
            position,
            attitude: 0.0,
        }
    }

    fn apply_thrust(&mut self) {
        self.velocity = self
                .velocity
                .translate(Transform::rotate(self.attitude) * (Vector::new(0, -30) / 60.0));
    }

    fn apply_gravity(&mut self) {
        self.velocity = self.velocity.translate(Vector::new(0, 10) / 60.0);
    }

    fn tick_position(&mut self) {
        self.position = self.position.translate(self.velocity / 60.0);
    }
}

impl Drawable for LunarModule {
    fn draw<'a>(&self, mesh: &mut Mesh, bkg: Background<'a>, trans: Transform, z: impl Scalar) {
        let top = Circle::new(self.position + (Transform::rotate(self.attitude) * Vector::new(0, -10)), 5);
        let top_black = Circle::new(self.position + (Transform::rotate(self.attitude) * Vector::new(0, -10)), 4);

        let main_rect_top_left = self.position + Vector::new(-5, -5);
        let main_rect = Rectangle::new(main_rect_top_left, Vector::new(10, 10));
        let black_rect_top_left = self.position + Vector::new(-4, -4);
        let black_rect = Rectangle::new(black_rect_top_left, Vector::new(8, 8));

        // feet

        let bottom_left = self.position + (Transform::rotate(self.attitude) * Vector::new(-5, 5));
        let left_leg_base = Line::new(bottom_left, bottom_left + (Transform::rotate(self.attitude) * Vector::new(-5, 5)));

        let bottom_right = self.position + (Transform::rotate(self.attitude) * Vector::new(5, 5));
        let right_leg_base = Line::new(bottom_right, bottom_right + (Transform::rotate(self.attitude) * Vector::new(5, 5)));

        main_rect.draw(mesh, bkg, Transform::rotate(self.attitude) * trans, z);
        black_rect.draw(mesh, Col(Color::BLACK), Transform::rotate(self.attitude) * trans, z);
        top.draw(mesh, bkg, trans, z);
        top_black.draw(mesh, Col(Color::BLACK), trans, z);
        left_leg_base.draw(mesh, bkg, trans, z);
        right_leg_base.draw(mesh, bkg, trans, z);
    }
}

struct Game {
    font: Asset<Font>,
    lunar_module: LunarModule,
    map: Asset<Map>,
}

impl State for Game {
    // Initialize the struct
    fn new() -> Result<Game> {
        let map = Asset::new(load_file("map.json")
             .and_then(|payload| {
                 Ok(from_utf8(&payload).unwrap().to_owned())
             })
             .and_then(|json_map| {
                 let mut map_message: MapMessage = serde_json::from_str(&json_map).unwrap();
                 Ok(map_message.extract_map())
             }));

        let font = Asset::new(Font::load("font.ttf"));

        Ok(Game {
            font: font,
            lunar_module: LunarModule::new(Vector::new(400, 300)),
            map: map,
        })
    }

    fn update(&mut self, window: &mut Window) -> Result<()> {
        if window.keyboard()[Key::Left].is_down() || window.keyboard()[Key::A].is_down(){
            self.lunar_module.attitude -= 3.0;
        }
        if window.keyboard()[Key::Right].is_down() || window.keyboard()[Key::D].is_down() {
            self.lunar_module.attitude += 3.0;
        }
        if window.keyboard()[Key::Up].is_down() || window.keyboard()[Key::W].is_down(){
            self.lunar_module.apply_thrust();
        }
        if window.keyboard()[Key::Space].is_down() {
            self.lunar_module = LunarModule::new(Vector::new(400, 300))
        }
        self.lunar_module.apply_gravity();
        self.lunar_module.tick_position();
        Ok(())
    }

    fn draw(&mut self, window: &mut Window) -> Result<()> {
        window.clear(Color::BLACK)?;

        self.map.execute(|map|{
            // draw map
            window.draw(map, Col(Color::WHITE));
            Ok(())
        })?;
        window.draw(&self.lunar_module, Color::WHITE);
        
        let horizontal = self.lunar_module.velocity.x;
        let vertical = self.lunar_module.velocity.y;
        self.font.execute(move |font| {
            let style = FontStyle::new(20.0, Color::WHITE);
            let text = format!("Horizontal: {:.0}\nVertical: {:.0}", horizontal, vertical);
            let image = font.render(&text, &style).unwrap();
            window.draw(&image.area().with_center((600, 100)), Img(&image));
            Ok(())
        })?;
        Ok(())
    }
}

fn main() {
    run::<Game>("Moon lander", Vector::new(800, 600), Settings::default());
}
