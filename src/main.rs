use quicksilver::{
    geom::{Line, Rectangle, Circle, Shape, Transform, Vector, Scalar},
    graphics::{
        Background::{Col, Img},
        Background,
        Color, Font, FontStyle, Drawable, Mesh, View
    },
    input::{Key, ButtonState},
    lifecycle::{run, Asset, Settings, State, Window},
    Result,
};
use serde::{Deserialize, Serialize};
use std::str::from_utf8;
use rand;

#[derive(Serialize, Deserialize)]
struct MapMessage {
    points: Vec<Vector>,
}

struct Map {
    lines: Vec<Line>,
}

impl MapMessage {
    fn extract_map(& self) -> Map {
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
    desired_attitude: f32,
    state: LunarModuleState,
    thruster_on: bool,
}

enum LunarModuleState {
    Flying,
    Landed,
    Crashed(CrashReason),
}

enum CrashReason {
    AngleTooSteep(f32),
    VelocityTooHigh(Vector),
    SurfaceNotFlat(Line),
}

impl LunarModule {
    fn new(position: Vector) -> LunarModule {
        LunarModule {
            velocity: Vector::new(0.0, 0.0),
            position,
            attitude: 0.0,
            desired_attitude: 0.0,
            state: LunarModuleState::Flying,
            thruster_on: false,
        }
    }

    fn update_attitude(&mut self) {
        if let LunarModuleState::Flying = self.state {
            self.attitude = self.desired_attitude;
        }
    }

    fn apply_thrust(&mut self) {
        if let LunarModuleState::Flying = self.state {
            self.velocity = self
                    .velocity
                    .translate(Transform::rotate(self.attitude) * (Vector::new(0, -15) / 60.0));
            self.thruster_on = true;
        }
    }

    fn disable_thrust(&mut self) {
        self.thruster_on = false;
    }

    fn tick_position(&mut self) {
        // gravity
        self.velocity = self.velocity.translate(Vector::new(0, 5) / 60.0);
        self.position = self.position.translate(self.velocity / 60.0);
    }

    fn check_collision(&mut self, map: &Map) {
        let top = Circle::new(self.position + (Transform::rotate(self.attitude) * Vector::new(0, -6)), 4);
        let main_rect = Rectangle::new(self.position + Vector::new(-5, -2), Vector::new(10, 4));
        let bottom_left = self.position + (Transform::rotate(self.attitude) * Vector::new(-3, 2));
        let left_leg_base = Line::new(bottom_left, bottom_left + (Transform::rotate(self.attitude) * Vector::new(-3, 2)));

        let bottom_right = self.position + (Transform::rotate(self.attitude) * Vector::new(3, 2));
        let right_leg_base = Line::new(bottom_right, bottom_right + (Transform::rotate(self.attitude) * Vector::new(3, 2)));


        for line in map.lines.iter() {
            let colliding = top.intersects(&line) || main_rect.intersects(&line) || left_leg_base.intersects(&line) || right_leg_base.intersects(&line);
            if colliding {
                if line.a.y == line.b.y && self.velocity.len() < 20.0 && self.attitude.abs() < 15.0 {
                    self.state = LunarModuleState::Landed;
                    return;
                } else {
                    self.disable_thrust();
                    if line.a.y != line.b.y {
                        self.state = LunarModuleState::Crashed(CrashReason::SurfaceNotFlat(line.clone()));
                    } else if self.velocity.len() > 20.0 {
                        self.state = LunarModuleState::Crashed(CrashReason::VelocityTooHigh(self.velocity.clone()));
                    } else if self.attitude.abs() > 15.0 {
                        self.state = LunarModuleState::Crashed(CrashReason::AngleTooSteep(self.attitude));
                    }
                    return;
                }
            }
        }
        self.state = LunarModuleState::Flying;
    }
}

impl Drawable for LunarModule {
    fn draw<'a>(&self, mesh: &mut Mesh, bkg: Background<'a>, trans: Transform, z: impl Scalar) {
        let top = Circle::new(self.position + (Transform::rotate(self.attitude) * Vector::new(0, -6)), 4);
        let top_black = Circle::new(self.position + (Transform::rotate(self.attitude) * Vector::new(0, -6)), 3);

        let main_rect = Rectangle::new(self.position + Vector::new(-5, -2), Vector::new(10, 4));
        let black_rect = Rectangle::new(self.position + Vector::new(-4, -1), Vector::new(8, 2));

        // feet

        let bottom_left = self.position + (Transform::rotate(self.attitude) * Vector::new(-3, 2));
        let left_leg_base = Line::new(bottom_left, bottom_left + (Transform::rotate(self.attitude) * Vector::new(-3, 2)));

        let bottom_right = self.position + (Transform::rotate(self.attitude) * Vector::new(3, 2));
        let right_leg_base = Line::new(bottom_right, bottom_right + (Transform::rotate(self.attitude) * Vector::new(3, 2)));

        let color = match self.state {
            LunarModuleState::Flying => bkg,
            LunarModuleState::Landed => Col(Color::GREEN),
            LunarModuleState::Crashed(_) => Col(Color::RED),
        };

        main_rect.draw(mesh, color, Transform::rotate(self.attitude) * trans, z);
        black_rect.draw(mesh, Col(Color::BLACK), Transform::rotate(self.attitude) * trans, z);
        top.draw(mesh, color, trans, z);
        top_black.draw(mesh, Col(Color::BLACK), trans, z);
        left_leg_base.draw(mesh, color, trans, z);
        right_leg_base.draw(mesh, color, trans, z);

        if self.thruster_on {
            // Fire
            let fire_dis = Transform::rotate(self.attitude) * Vector::new(-2.0 + rand::random::<f32>() * 4.0, 17.0 + rand::random::<f32>() * 4.0);
            let left_fire = Line::new(bottom_left, fire_dis + self.position);
            let right_fire = Line::new(bottom_right, fire_dis + self.position);
            left_fire.draw(mesh, Col(Color::RED), trans, z);
            right_fire.draw(mesh, Col(Color::RED), trans, z);
        }
    }
}

struct Game {
    font: Asset<Font>,
    lunar_module: LunarModule,
    map: Map,
    view_rectangle: Option<Rectangle>,
}

impl State for Game {
    // Initialize the struct
    fn new() -> Result<Game> {
        let map_payload = include_bytes!("map.json");
        let map_json = from_utf8(map_payload).unwrap();
        let map_message: MapMessage = serde_json::from_str(map_json).unwrap();
        let map = map_message.extract_map();

        let font = Asset::new(Font::load("font.ttf"));

        Ok(Game {
            font: font,
            lunar_module: LunarModule::new(Vector::new(400, 300)),
            map: map,
            view_rectangle: None,
        })
    }

    fn update(&mut self, window: &mut Window) -> Result<()> {
        if window.keyboard()[Key::Left].is_down() || window.keyboard()[Key::A].is_down(){
            self.lunar_module.desired_attitude -= 2.5;
        }
        if window.keyboard()[Key::Right].is_down() || window.keyboard()[Key::D].is_down() {
            self.lunar_module.desired_attitude += 2.5;
        }
        self.lunar_module.update_attitude();
        if window.keyboard()[Key::Up].is_down() || window.keyboard()[Key::W].is_down(){
            self.lunar_module.apply_thrust();
        } else {
            self.lunar_module.disable_thrust();
        }
        if window.keyboard()[Key::Space].is_down() {
            self.lunar_module = LunarModule::new(Vector::new(400, 300))
        }
        if window.keyboard()[Key::F] == ButtonState::Pressed {
            window.set_fullscreen(!window.get_fullscreen());
        }
        self.lunar_module.check_collision(&self.map);
        if let LunarModuleState::Flying = self.lunar_module.state {
                self.lunar_module.tick_position();
        }

        let top_left = self.lunar_module.position - Vector::new(50, 50);
        let detailed_view_rectangle = Rectangle::new(top_left, Vector::new(100, 100));
        let mut detailed_view_needed = false;
        for line in self.map.lines.iter() {
            if detailed_view_rectangle.overlaps(line) {
                self.view_rectangle = Some(detailed_view_rectangle);
                let new_view = View::new(detailed_view_rectangle);
                window.set_view(new_view);
                detailed_view_needed = true;
                break;
            }
        }

        if !detailed_view_needed {
            self.view_rectangle = None;
            let screen_size = window.screen_size();
            let view_rectangle = Rectangle::new(Vector::new(0, 0), screen_size);
            let new_view = View::new(view_rectangle);
            window.set_view(new_view);
        }

        Ok(())
    }

    fn draw(&mut self, window: &mut Window) -> Result<()> {
        window.clear(Color::BLACK)?;

        window.draw(&self.map, Col(Color::WHITE));

        window.draw(&self.lunar_module, Color::WHITE);
        
        let horizontal = self.lunar_module.velocity.x;
        let vertical = self.lunar_module.velocity.y;
        let text_point = match self.view_rectangle {
            None => Vector::new(600, 100),
            Some(rectangle) => {
                rectangle.top_left() + Vector::new( rectangle.size().x * 0.8, rectangle.size().y / 10.0)
            }
        };
        let scale = match self.view_rectangle {
            None => false,
            Some(_) => true,
        };
        self.font.execute(move |font| {
            let style = FontStyle::new(20.0, Color::WHITE);
            let text = format!("Horizontal: {:.0}\nVertical: {:.0}", horizontal, vertical);
            let image = font.render(&text, &style).unwrap();
            if scale {
                window.draw_ex(&image.area().with_center(text_point), Img(&image), Transform::scale(Vector::new(0.25, 0.25)), 10);
            } else {
                window.draw(&image.area().with_center(text_point), Img(&image));
            }
            Ok(())
        })?;

        Ok(())
    }
}

fn main() {
    run::<Game>("Moon lander", Vector::new(800, 600), Settings::default());
}
