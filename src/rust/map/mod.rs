use collisions;
use level_serialization;

#[derive(Debug, Copy, Clone)]
enum PlatformType {
    Box,
    Line,
}

#[derive(Debug, Copy, Clone)]
struct Platform {
    min_x: f64,
    min_y: f64,
    len_x: f64,
    len_y: f64,
    platform_type: PlatformType,
}

impl collisions::HasBounds for Platform {
    fn min_x(&self) -> f64 {
        self.min_x
    }

    fn min_y(&self) -> f64 {
        self.min_y
    }

    fn len_x(&self) -> f64 {
        self.len_x
    }

    fn len_y(&self) -> f64 {
        self.len_y
    }
}

pub struct Map {
    blocks: Vec<Platform>,
    initial_x: f64,
    initial_y: f64,
}

impl Map {
    pub fn blocks(&self) -> &[Platform] {
        &self.blocks
    }

    pub fn initial_x(&self) -> f64 {
        self.initial_x
    }

    pub fn initial_y(&self) -> f64 {
        self.initial_y
    }
}

impl From<level_serialization::Level> for Map {
    fn from(level: level_serialization::Level) -> Map {
        let blocks = level.items.into_iter().map(|item| {
            match item {
                level_serialization::LevelItem::Box { x, y, width, height } => {
                    Platform { min_x: x, min_y: y, len_x: width, len_y: height,
                                platform_type: PlatformType::Box }
                },
                level_serialization::LevelItem::Line { x, y, direction, length } => {
                    match direction {
                        level_serialization::Direction::North => {
                            Platform { min_x: x, min_y: y, len_x: 1.0, len_y: length,
                                platform_type: PlatformType::Line }
                        },
                        level_serialization::Direction::East => {
                            Platform { min_x: x, min_y: y, len_x: length, len_y: 1.0,
                                platform_type: PlatformType::Line }
                        }
                    }
                },
            }
        }).collect();

        println!("{:#?}", blocks);

        Map {
            blocks: blocks,
            initial_x: level.initial_x,
            initial_y: level.initial_y,
        }
    }
}
