use quicksilver::{
    geom::{Line, Rectangle, Circle, Shape, Transform, Vector, Scalar},
    graphics::{
        Background::Col,
        Background,
        Color,
        Drawable,
        Mesh,
    },
};
use std::time::Instant;

use crate::map::Map;

const ZOOMED_SCALE: f32 = 0.6;

pub struct LunarModule {
    pub velocity: Vector,
    pub position: Vector,
    pub desired_attitude: f32,
    pub state: LunarModuleState,
    pub zoomed: bool,
    attitude: f32,
    thruster_on: bool,
    thruster_on_time: Instant,
}

#[derive(Clone)]
pub enum LunarModuleState {
    Flying,
    Landed,
    Crashed(CrashReason),
}

#[derive(Clone)]
pub enum CrashReason {
    AngleTooSteep(f32),
    VelocityTooHigh(Vector),
    SurfaceNotFlat(Line),
}

impl LunarModule {
    pub fn new() -> LunarModule {
        LunarModule {
            velocity: Vector::new(20.0 + rand::random::<f32>() * 20.0, 0.0),
            position: Vector::new(400, 300),
            attitude: 90.0,
            desired_attitude: 90.0,
            state: LunarModuleState::Flying,
            thruster_on: true,
            zoomed: false,
            thruster_on_time: Instant::now(),
        }
    }

    pub fn reset(&mut self) {
        self.velocity = Vector::new(20.0 + rand::random::<f32>() * 20.0, 0.0);
        self.position = Vector::new(400, 300);
        self.attitude = 90.0;
        self.desired_attitude = 90.0;
        self.state = LunarModuleState::Flying;
        self.thruster_on = true;
        self.thruster_on_time = Instant::now();
    }

    pub fn update_attitude(&mut self) {
        if let LunarModuleState::Flying = self.state {
            self.attitude = self.desired_attitude;
        }
    }

    pub fn apply_thrust(&mut self) {
        if let LunarModuleState::Flying = self.state {
            self.velocity = self
                    .velocity
                    .translate(Transform::rotate(self.attitude) * (Vector::new(0, -15) / 60.0));
            if !self.thruster_on {
                self.thruster_on_time = Instant::now();
            }
            self.thruster_on = true;
        }
    }

    pub fn disable_thrust(&mut self) {
        self.thruster_on = false;
    }

    pub fn tick_position(&mut self) {
        // gravity
        self.velocity = self.velocity.translate(Vector::new(0, 5) / 60.0);
        self.position = self.position.translate(self.velocity / 60.0);
    }

    pub fn check_collision(&mut self, map: &Map) {
        let multiplier = if self.zoomed { ZOOMED_SCALE } else { 1.0 };
        
        let top = Circle::new(self.position + (Transform::rotate(self.attitude) * Vector::new(0, -6) * multiplier), 4.0 * multiplier);
        // feet
        let bottom_left = self.position + (Transform::rotate(self.attitude) * Vector::new(-3, 2) * multiplier);
        let left_leg_base = Line::new(bottom_left, bottom_left + (Transform::rotate(self.attitude) * Vector::new(-3, 2) * multiplier));
        let bottom_right = self.position + (Transform::rotate(self.attitude) * Vector::new(3, 2) * multiplier);
        let right_leg_base = Line::new(bottom_right, bottom_right + (Transform::rotate(self.attitude) * Vector::new(3, 2) * multiplier));

        for line in map.lines.iter() {
            let colliding = top.intersects(&line) || left_leg_base.intersects(&line) || right_leg_base.intersects(&line);
            if colliding {
                self.disable_thrust();
                let attitude = (self.attitude % 360.0).abs();
                if self.velocity.len() > 20.0 {
                    self.state = LunarModuleState::Crashed(CrashReason::VelocityTooHigh(self.velocity.clone()));
                } else if line.a.y != line.b.y {
                    self.state = LunarModuleState::Crashed(CrashReason::SurfaceNotFlat(line.clone()));
                } else if attitude > 10.0 && attitude < 360.0 - 10.0 {
                    self.state = LunarModuleState::Crashed(CrashReason::AngleTooSteep(attitude));
                } else {
                    self.state = LunarModuleState::Landed;
                }
                return;
            }
        }
        self.state = LunarModuleState::Flying;
    }
}

impl Drawable for LunarModule {
    fn draw<'a>(&self, mesh: &mut Mesh, bkg: Background<'a>, trans: Transform, z: impl Scalar) {

        let multiplier = if self.zoomed { ZOOMED_SCALE } else { 1.0 };

        let top = Circle::new(self.position + (Transform::rotate(self.attitude) * Vector::new(0, -6) * multiplier), 4.0 * multiplier);
        let top_black = Circle::new(self.position + (Transform::rotate(self.attitude) * Vector::new(0, -6) * multiplier), 3.0 * multiplier);

        let main_rect = Rectangle::new(self.position + Vector::new(-5, -2) * multiplier, Vector::new(10, 4) * multiplier);
        let black_rect = Rectangle::new(self.position + Vector::new(-4, -1) * multiplier, Vector::new(8, 2) * multiplier);

        // feet
        let bottom_left = self.position + (Transform::rotate(self.attitude) * Vector::new(-3, 2) * multiplier);
        let left_leg_base = Line::new(bottom_left, bottom_left + (Transform::rotate(self.attitude) * Vector::new(-3, 2) * multiplier));
        let bottom_right = self.position + (Transform::rotate(self.attitude) * Vector::new(3, 2) * multiplier);
        let right_leg_base = Line::new(bottom_right, bottom_right + (Transform::rotate(self.attitude) * Vector::new(3, 2) * multiplier));

        // different colors were used for debug
        // let color = match self.state {
        //     LunarModuleState::Flying => bkg,
        //     LunarModuleState::Landed => Col(Color::GREEN),
        //     LunarModuleState::Crashed(_) => Col(Color::RED),
        // };
        let color = bkg;

        main_rect.draw(mesh, color, Transform::rotate(self.attitude) * trans, z);
        black_rect.draw(mesh, Col(Color::BLACK), Transform::rotate(self.attitude) * trans, z);
        top.draw(mesh, color, trans, z);
        top_black.draw(mesh, Col(Color::BLACK), trans, z);
        left_leg_base.draw(mesh, color, trans, z);
        right_leg_base.draw(mesh, color, trans, z);

        if self.thruster_on {
            let scale = (self.thruster_on_time.elapsed().as_secs_f32() * 4.0).min(1.0);
            // Fire
            let fire_dis = Transform::rotate(self.attitude) * Vector::new(0.0, 19.0 + rand::random::<f32>() * 4.0) * multiplier * scale;
            let left_fire = Line::new(bottom_left, fire_dis + self.position);
            let right_fire = Line::new(bottom_right, fire_dis + self.position);
            left_fire.draw(mesh, Col(Color::RED), trans, z);
            right_fire.draw(mesh, Col(Color::RED), trans, z);
        }
    }
}
