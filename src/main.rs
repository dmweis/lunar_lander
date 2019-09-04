// Demonstrate adding a View to the draw-geometry example
// The camera can be controlled with the arrow keys
use lazy_static;
use quicksilver::{
    combinators::result,
    geom::{Line, Rectangle, Shape, Transform, Vector},
    graphics::{
        Background::{Col, Img},
        Color, Font, FontStyle, Image,
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
    fn new(points: Vec<Vector>) -> Map {
        Map { points }
    }

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

// lazy_static::lazy_static! {
//     static ref MAP: Vec<Line> = {
//                 let payload = load_file("map.json").wait();
//                 // let json_map = from_utf8(&payload).expect("second");
//                 // let map: Map= serde_json::from_str(json_map).expect("third");
//                 let map = Map::new(vec![Vector::new(0, 0), Vector::new(100, 100)]);
//                 map.get_lines()
//             };

// }

struct Game {
    text_render_counter: u32,
    text: Asset<Image>,
    velocity: Vector,
    position: Vector,
    angle: i32,
    map: Asset<Vec<Line>>,
}

impl State for Game {
    // Initialize the struct
    fn new() -> Result<Game> {
        let text = Asset::new(Font::load("font.ttf").and_then(|font| {
            let style = FontStyle::new(20.0, Color::WHITE);
            result(font.render("Velocity: fast", &style))
        }));

        let map = Asset::new(load_file("map.json")
             .and_then(|payload| {
                 Ok(from_utf8(&payload).unwrap().to_owned())
             })
             .and_then(|json_map| {
                 let map: Map = serde_json::from_str(&json_map).unwrap();
                 Ok(map.get_lines())
             }));

        Ok(Game {
            text_render_counter: 0,
            text: text,
            velocity: Vector::new(0, 0),
            position: Vector::new(400, 300),
            angle: 0,
            map: map,
        })
    }

    fn update(&mut self, window: &mut Window) -> Result<()> {
        if window.keyboard()[Key::Left].is_down() {
            self.angle = self.angle - 3;
        }
        if window.keyboard()[Key::Right].is_down() {
            self.angle = self.angle + 3;
        }
        if window.keyboard()[Key::Up].is_down() {
            self.velocity = self
                .velocity
                .translate(Transform::rotate(self.angle) * (Vector::new(0, -30) / 60.0));
        }
        self.velocity = self.velocity.translate(Vector::new(0, 10) / 60.0);
        self.position = self.position.translate(self.velocity / 60.0);

        let horizontal = self.velocity.x;
        let vertical = self.velocity.y;
        // render text
        self.text_render_counter += 1;
        if self.text_render_counter >= 30 {
            self.text_render_counter = 0;
            self.text = Asset::new(Font::load("font.ttf").and_then(move |font| {
                let style = FontStyle::new(20.0, Color::WHITE);
                let text = format!("Horizontal: {}\nVertical {}", horizontal, vertical);
                result(font.render(&text, &style))
            }));
        }
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
        self.text.execute(|image| {
            window.draw(&image.area().with_center((600, 50)), Img(&image));
            Ok(())
        })?;
        Ok(())
    }
}

fn main() {
    run::<Game>("Moon lander", Vector::new(800, 600), Settings::default());
}
