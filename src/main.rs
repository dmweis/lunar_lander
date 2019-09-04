use quicksilver::{
    geom::{Line, Rectangle, Shape, Transform, Vector},
    graphics::{
        Background::{Col, Img},
        Color, Font, FontStyle,
    },
    input::Key,
    lifecycle::{run, Asset, Settings, State, Window},
    load_file, Future, Result,
};
use serde::{Deserialize, Serialize};
use std::str::from_utf8;

#[derive(Serialize, Deserialize)]
struct Map {
    points: Vec<Vector>,
}

impl Map {
    fn get_lines(self) -> Vec<Line> {
        let mut lines: Vec<Line> = Vec::new();
        let mut last_point = self.points[0];
        for point in self.points.into_iter().skip(1) {
            lines.push(Line::new(last_point, point));
            last_point = point;
        }
        lines
    }
}

struct Game {
    font: Asset<Font>,
    velocity: Vector,
    position: Vector,
    angle: i32,
    map: Asset<Vec<Line>>,
}

impl State for Game {
    // Initialize the struct
    fn new() -> Result<Game> {
        let map = Asset::new(load_file("map.json")
             .and_then(|payload| {
                 Ok(from_utf8(&payload).unwrap().to_owned())
             })
             .and_then(|json_map| {
                 let map: Map = serde_json::from_str(&json_map).unwrap();
                 Ok(map.get_lines())
             }));

        let font = Asset::new(Font::load("font.ttf"));

        Ok(Game {
            font: font,
            velocity: Vector::new(0, 0),
            position: Vector::new(400, 300),
            angle: 0,
            map: map,
        })
    }

    fn update(&mut self, window: &mut Window) -> Result<()> {
        if window.keyboard()[Key::Left].is_down() || window.keyboard()[Key::A].is_down(){
            self.angle = self.angle - 3;
        }
        if window.keyboard()[Key::Right].is_down() || window.keyboard()[Key::D].is_down() {
            self.angle = self.angle + 3;
        }
        if window.keyboard()[Key::Up].is_down() || window.keyboard()[Key::W].is_down(){
            self.velocity = self
                .velocity
                .translate(Transform::rotate(self.angle) * (Vector::new(0, -30) / 60.0));
        }
        self.velocity = self.velocity.translate(Vector::new(0, 10) / 60.0);
        self.position = self.position.translate(self.velocity / 60.0);

        Ok(())
    }

    fn draw(&mut self, window: &mut Window) -> Result<()> {
        window.clear(Color::BLACK)?;

        self.map.execute(|map|{
            // draw map
            for line in map.iter() {
                window.draw(line, Col(Color::WHITE));
            }
            Ok(())
        })?;

        let top_left = self.position - Vector::new(-5, -5);

        let player_model = Rectangle::new(top_left, Vector::new(10, 10));
        window.draw_ex(
            &player_model,
            Col(Color::WHITE),
            Transform::rotate(self.angle),
            10,
        );
        let horizontal = self.velocity.x;
        let vertical = self.velocity.y;
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
