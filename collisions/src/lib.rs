use std::f64;

#[derive(Copy, Clone, Debug, Default)]
pub struct Collisions {
    pub north: Option<f64>,
    pub south: Option<f64>,
    pub east: Option<f64>,
    pub west: Option<f64>,
}

fn collides(x1: f64, y1: f64, width1: f64, height1: f64,
            x2: f64, y2: f64, width2: f64, height2: f64) -> bool {
    x1 <= x2 + width2 &&
    x1 + width1 >= x2 &&
    y1 <= y2 + height2 &&
    y1 + height1 >= y2
}

fn collides1d(x1: f64, width1: f64, x2: f64, width2: f64) -> bool {
    x1 < x2 + width2 &&
    x1 + width1 > x2
}

fn within(bound_start: f64, bound_length: f64, value: f64) -> bool {
    value <= bound_start + bound_length && value >= bound_start
}

fn now_past(point: f64, old_pos: f64, new_pos: f64, length: f64) -> bool {
    new_pos + length > point && old_pos + length <= point
}

fn now_before(point: f64, old_pos: f64, new_pos: f64) -> bool {
    new_pos < point && old_pos >= point
}

fn min_f64(v1: Option<f64>, v2: Option<f64>) -> Option<f64> {
    match (v1, v2) {
        (None, None) => None,
        (Some(x), None) => Some(x),
        (None, Some(x)) => Some(x),
        (Some(x), Some(y)) => Some(f64::min(x, y)),
    }
}

fn max_f64(v1: Option<f64>, v2: Option<f64>) -> Option<f64> {
    match (v1, v2) {
        (None, None) => None,
        (Some(x), None) => Some(x),
        (None, Some(x)) => Some(x),
        (Some(x), Some(y)) => Some(f64::max(x, y)),
    }
}


pub trait HasBounds {
    fn min_x(&self) -> f64;
    fn min_y(&self) -> f64;
    fn len_x(&self) -> f64;
    fn len_y(&self) -> f64;

    fn collides<'a, T: ?Sized, I>(&self, next_x: f64, next_y: f64, blocks: I) -> Collisions
        where T: HasBounds + std::fmt::Debug + 'a, I: IntoIterator<Item=&'a T> {

        let mut collisions = Collisions::default();

        for block in blocks.into_iter() {
            if !collides(next_x, next_y, self.len_x(), self.len_y(),
                        block.min_x(), block.min_y(), block.len_x(), block.len_y()) {
                continue;
            }

            if collides1d(self.min_x(), self.len_x(), block.min_x(), block.len_x()) {
                match (within(block.min_y(), block.len_y(), next_y + self.len_y())
                        || now_past(block.min_y(), self.min_y(), next_y, self.len_y()),
                        within(block.min_y(), block.len_y(), next_y)
                        || now_before(block.min_y() + block.len_y(), self.min_y(), next_y)) {
                    (true, false) => {
                        collisions.north = min_f64(collisions.north,
                            Some(block.min_y() - self.len_y()));
                    },
                    (false, true) => {
                        collisions.south = max_f64(collisions.south,
                            Some(block.min_y() + block.len_y()));
                    },
                    (_, _) => (),
                }
            }
            if collides1d(self.min_y(), self.len_y(), block.min_y(), block.len_y()) {
                match (within(block.min_x(), block.len_x(), next_x + self.len_x())
                        || now_past(block.min_x(), self.min_x(), next_x, self.len_x()),
                        within(block.min_x(), block.len_x(), next_x)
                        || now_before(block.min_x() + block.len_x(), self.min_x(), next_x)) {
                    (true, false) => {
                        collisions.east = max_f64(collisions.east,
                            Some(block.min_x() - self.len_x()));
                    },
                    (false, true) => {
                        collisions.west = min_f64(collisions.west,
                            Some(block.min_x() + block.len_x()));
                    },
                    (_, _) => (),
                }
            }
        }

        collisions
    }
}
