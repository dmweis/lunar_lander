use quicksilver::{
    geom::{Line, Rectangle, Circle, Shape, Transform, Vector, Scalar},
    graphics::{
        Background::{Col, Img},
        Background,
        Color, Font, FontStyle, Drawable, Mesh, View, ResizeStrategy,
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
            let point  = point;
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

#[derive(Clone)]
enum LunarModuleState {
    Flying,
    Landed,
    Crashed(CrashReason),
}

#[derive(Clone)]
enum CrashReason {
    AngleTooSteep(f32),
    VelocityTooHigh(Vector),
    SurfaceNotFlat(Line),
}

impl LunarModule {
    fn new() -> LunarModule {
        LunarModule {
            velocity: Vector::new(20.0 + rand::random::<f32>() * 20.0, 0.0),
            position: Vector::new(400, 300),
            attitude: 90.0,
            desired_attitude: 90.0,
            state: LunarModuleState::Flying,
            thruster_on: true,
        }
    }

    fn reset(&mut self) {
        self.velocity = Vector::new(20.0 + rand::random::<f32>() * 20.0, 0.0);
        self.position = Vector::new(400, 300);
        self.attitude = 90.0;
        self.desired_attitude = 90.0;
        self.state = LunarModuleState::Flying;
        self.thruster_on = true;
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
                if line.a.y == line.b.y && self.velocity.len() < 20.0 && (self.attitude < 15.0 || self.attitude > 350.0) {
                    self.state = LunarModuleState::Landed;
                    return;
                } else {
                    self.disable_thrust();
                    if line.a.y != line.b.y {
                        self.state = LunarModuleState::Crashed(CrashReason::SurfaceNotFlat(line.clone()));
                    } else if self.velocity.len() > 20.0 {
                        self.state = LunarModuleState::Crashed(CrashReason::VelocityTooHigh(self.velocity.clone()));
                    } else if self.attitude > 15.0 || self.attitude < 350.0 {
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
            let fire_dis = Transform::rotate(self.attitude) * Vector::new(0.0, 17.0 + rand::random::<f32>() * 6.0);
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
    fullscreen: bool,
    started: bool
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
            lunar_module: LunarModule::new(),
            map: map,
            view_rectangle: None,
            fullscreen: false,
            started: false,
        })
    }

    fn update(&mut self, window: &mut Window) -> Result<()> {
        if window.get_fullscreen() != self.fullscreen {
            window.set_fullscreen(self.fullscreen);
        }
        if !self.started {
            if window.keyboard()[Key::Space].is_down() {
                self.started = true;
            }
        } else {
            if window.keyboard()[Key::Escape].is_down(){
                self.lunar_module.reset();
                self.started = false;
            }
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
                self.lunar_module.reset();
            }
            if window.keyboard()[Key::F] == ButtonState::Pressed {
                self.fullscreen = !self.fullscreen;
            }
            self.lunar_module.check_collision(&self.map);
            if let LunarModuleState::Flying = self.lunar_module.state {
                    self.lunar_module.tick_position();
            }
        }

        let top_left = self.lunar_module.position - Vector::new(300, 100);
        let detailed_view_checker = Rectangle::new(top_left, Vector::new(400, 200));
        let detailed_view_rectangle = Rectangle::new(top_left, Vector::new(600, 300));
        let mut detailed_view_needed = false;
        for line in self.map.lines.iter() {
            if detailed_view_checker.overlaps(line) {
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
        let game_state = self.lunar_module.state.clone();
        let started = self.started;
        let zoomed = match self.view_rectangle {
            None => false,
            Some(_) => true,
        };
        let view_rectangle = match self.view_rectangle {
            None => Rectangle::new(Vector::new(0, 0), window.screen_size()),
            Some(rectangle) => rectangle,
        };

        self.font.execute(move |font| {
            if !started {
                let style = FontStyle::new(60.0, Color::WHITE);
                let text = "Welcome!\nUse WASD or arrow keys to control\nPress Space to star game";
                let image = font.render(&text, &style).unwrap();
                let text_point = view_rectangle.top_left() + Vector::new(view_rectangle.size().x * 0.5, view_rectangle.size().y * 0.2);
                window.draw_ex(&image.area().with_center(text_point), Img(&image), Transform::scale(Vector::new(0.5, 0.5)), 10);
            }

            let style = FontStyle::new(60.0, Color::WHITE);
            let text = format!("Horizontal: {:.0}\nVertical: {:.0}", horizontal, vertical);
            let image = font.render(&text, &style).unwrap();
            let text_point = view_rectangle.top_left() + Vector::new(view_rectangle.size().x * 0.8, view_rectangle.size().y / 10.0);
            if zoomed {
                window.draw_ex(&image.area().with_center(text_point), Img(&image), Transform::scale(Vector::new(0.125, 0.125)), 10);
            } else {
                window.draw_ex(&image.area().with_center(text_point), Img(&image), Transform::scale(Vector::new(0.5, 0.5)), 10);
            }
            if let LunarModuleState::Crashed(reason) = game_state {
                // Draw info screen
                let style = FontStyle::new(60.0, Color::WHITE);
                let text = "Game Over!\nUse WASD or arrow keys to control\nPress Space to restart game";
                let image = font.render(&text, &style).unwrap();
                let text_point = view_rectangle.top_left() + Vector::new(view_rectangle.size().x * 0.5, view_rectangle.size().y * 0.1);
                window.draw_ex(&image.area().with_center(text_point), Img(&image), Transform::scale(Vector::new(0.2, 0.2)), 10);
                // draw crash reason
                let style = FontStyle::new(60.0, Color::RED);
                let text = match reason {
                    CrashReason::AngleTooSteep(angle) => format!("Angle too steep: {:.0}", angle),
                    CrashReason::VelocityTooHigh(vector) => format!("Velocity too high: {:.1}", vector.len()),
                    CrashReason::SurfaceNotFlat(_) => format!("Surface not flat"),
                };
                let image = font.render(&text, &style).unwrap();
                let screen_middle =  view_rectangle.top_left() + Vector::new(view_rectangle.size().x * 0.5, view_rectangle.size().y * 0.25);
                window.draw_ex(&image.area().with_center(screen_middle), Img(&image), Transform::scale(Vector::new(0.125, 0.125)), 10);
            }
            Ok(())
        })?;

        Ok(())
    }
}

fn main() {
    let mut settings =  Settings::default();
    settings.resize = ResizeStrategy::Fit;
    settings.draw_rate = 30.0;
    settings.fullscreen = true;

    run::<Game>("Moon lander", Vector::new(2000, 1000), settings);
}
