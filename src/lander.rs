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
use crate::map::Map;

pub struct LunarModule {
    pub velocity: Vector,
    pub position: Vector,
    attitude: f32,
    pub desired_attitude: f32,
    pub state: LunarModuleState,
    thruster_on: bool,
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
        }
    }

    pub fn reset(&mut self) {
        self.velocity = Vector::new(20.0 + rand::random::<f32>() * 20.0, 0.0);
        self.position = Vector::new(400, 300);
        self.attitude = 90.0;
        self.desired_attitude = 90.0;
        self.state = LunarModuleState::Flying;
        self.thruster_on = true;
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
        let top = Circle::new(self.position + (Transform::rotate(self.attitude) * Vector::new(0, -6)), 4);
        let main_rect = Rectangle::new(self.position + Vector::new(-5, -2), Vector::new(10, 4));

        // feet
        let bottom_left = self.position + (Transform::rotate(self.attitude) * Vector::new(-3, 2));
        let left_leg_base = Line::new(bottom_left, bottom_left + (Transform::rotate(self.attitude) * Vector::new(-3, 2)));
        let bottom_right = self.position + (Transform::rotate(self.attitude) * Vector::new(3, 2));
        let right_leg_base = Line::new(bottom_right, bottom_right + (Transform::rotate(self.attitude) * Vector::new(3, 2)));

        for line in map.lines.iter() {
            let colliding = top.intersects(&line) || main_rect.intersects(&line) || left_leg_base.intersects(&line) || right_leg_base.intersects(&line);
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
        let top = Circle::new(self.position + (Transform::rotate(self.attitude) * Vector::new(0, -6)), 4);
        let top_black = Circle::new(self.position + (Transform::rotate(self.attitude) * Vector::new(0, -6)), 3);

        let main_rect = Rectangle::new(self.position + Vector::new(-5, -2), Vector::new(10, 4));
        let black_rect = Rectangle::new(self.position + Vector::new(-4, -1), Vector::new(8, 2));

        // feet
        let bottom_left = self.position + (Transform::rotate(self.attitude) * Vector::new(-3, 2));
        let left_leg_base = Line::new(bottom_left, bottom_left + (Transform::rotate(self.attitude) * Vector::new(-3, 2)));
        let bottom_right = self.position + (Transform::rotate(self.attitude) * Vector::new(3, 2));
        let right_leg_base = Line::new(bottom_right, bottom_right + (Transform::rotate(self.attitude) * Vector::new(3, 2)));

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
            // Fire
            let fire_dis = Transform::rotate(self.attitude) * Vector::new(0.0, 17.0 + rand::random::<f32>() * 6.0);
            let left_fire = Line::new(bottom_left, fire_dis + self.position);
            let right_fire = Line::new(bottom_right, fire_dis + self.position);
            left_fire.draw(mesh, Col(Color::RED), trans, z);
            right_fire.draw(mesh, Col(Color::RED), trans, z);
        }
    }
}
