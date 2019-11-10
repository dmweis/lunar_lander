use quicksilver::{
    geom::Vector,
    graphics::ResizeStrategy,
    lifecycle::{run, Settings},
};

mod lander;
mod map;
mod game;

use game::Game;

fn main() {
    let mut settings =  Settings::default();
    settings.resize = ResizeStrategy::Fit;
    settings.draw_rate = 30.0;
    settings.fullscreen = true;
    run::<Game>("Moon lander", Vector::new(1600, 800), settings);
}
