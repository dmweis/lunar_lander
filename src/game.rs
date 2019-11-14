use quicksilver::{
    geom::{Rectangle, Shape, Transform, Vector},
    graphics::{
        Background::{Col, Img},
        Color, Font, FontStyle, View,
    },
    input::{Key, ButtonState},
    lifecycle::{Asset, State, Window},
    Result,
};
use std::str::from_utf8;

use crate::lander::{LunarModule, LunarModuleState, CrashReason};
use crate::map::{Map, MapMessage};

pub struct Game {
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
        let font = Asset::new(Font::load("ShareTechMono-Regular.ttf"));
        Ok(Game {
            font: font,
            lunar_module: LunarModule::new(),
            map: map,
            view_rectangle: None,
            fullscreen: true,
            started: false,
        })
    }

    fn update(&mut self, window: &mut Window) -> Result<()> {
        if window.keyboard()[Key::F] == ButtonState::Pressed {
            self.fullscreen = !self.fullscreen;
        }
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
            if window.keyboard()[Key::Up].is_down() || window.keyboard()[Key::W].is_down() || window.keyboard()[Key::Space].is_down(){
                self.lunar_module.apply_thrust();
            } else {
                self.lunar_module.disable_thrust();
            }
            if window.keyboard()[Key::R].is_down() {
                self.lunar_module.reset();
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
        self.lunar_module.zoomed = detailed_view_needed;
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
                let style = FontStyle::new(60.0, Color::MAGENTA);
                let text = "Welcome!\nUse WASD or arrow keys to control\nPress Space to star game";
                let image = font.render(&text, &style).unwrap();
                let text_point = view_rectangle.top_left() + Vector::new(view_rectangle.size().x * 0.5, view_rectangle.size().y * 0.2);
                window.draw_ex(&image.area().with_center(text_point), Img(&image), Transform::scale(Vector::new(0.5, 0.5)), 10);
            }

            // landed
            if let LunarModuleState::Landed = game_state {
                let style = FontStyle::new(60.0, Color::WHITE);
                let text = "Congratulations!\nYou landed successfully\nPress R to restart game";
                let image = font.render(&text, &style).unwrap();
                let text_point = view_rectangle.top_left() + Vector::new(view_rectangle.size().x * 0.5, view_rectangle.size().y * 0.1);
                window.draw_ex(&image.area().with_center(text_point), Img(&image), Transform::scale(Vector::new(0.2, 0.2)), 10);
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
                let text = "Game Over!\nUse WASD or arrow keys to control\nPress R to restart game";
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
