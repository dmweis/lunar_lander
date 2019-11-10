use quicksilver::{
    geom::{Line, Transform, Vector, Scalar},
    graphics::{
        Background,
        Drawable,
        Mesh,
    },
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct MapMessage {
    points: Vec<Vector>,
}

pub struct Map {
    pub lines: Vec<Line>,
}

impl MapMessage {
    pub fn extract_map(& self) -> Map {
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
