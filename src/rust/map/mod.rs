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

/// graphics::Rectangle
impl<'a> Into<[f64; 4]> for &'a Platform {
    fn into(self) -> [f64; 4] {
        [self.min_x, self.min_y, self.len_x, self.len_y]
    }
}

impl Into<[f64; 4]> for Platform {
    fn into(self) -> [f64; 4] {
        [self.min_x, self.min_y, self.len_x, self.len_y]
    }
}

pub struct Map {
    blocks: Vec<Platform>,
    boundary_collision_lines: Vec<Platform>,
    initial_x: f64,
    initial_y: f64,
    /// [west, south, east - west, north - south]
    boundaries: [f64; 4],
}

impl Map {
    pub fn blocks(&self) -> &[Platform] {
        &self.blocks
    }

    pub fn boundary_collision_lines(&self) -> &[Platform] {
        &self.boundary_collision_lines
    }

    pub fn initial_x(&self) -> f64 {
        self.initial_x
    }

    pub fn initial_y(&self) -> f64 {
        self.initial_y
    }

    /// [west, south, east - west, north - south]
    pub fn boundaries(&self) -> [f64; 4] {
        self.boundaries
    }
}

impl<'a> From<&'a level_serialization::Level> for Map {
    fn from(level: &'a level_serialization::Level) -> Map {
        let blocks = level.items.iter().map(|item| {
            match *item {
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

        let mut boundary_collision_lines = Vec::new();
        // West
        boundary_collision_lines.push(Platform {
            min_x: level.west_boundary - 1.0,
            min_y: level.south_boundary,
            len_x: 1.0,
            len_y: level.north_boundary - level.south_boundary,
            platform_type: PlatformType::Line,
        });
        // North
        boundary_collision_lines.push(Platform {
            min_x: level.west_boundary,
            min_y: level.north_boundary,
            len_x: level.east_boundary - level.west_boundary,
            len_y: 1.0,
            platform_type: PlatformType::Line,
        });
        // South
        boundary_collision_lines.push(Platform {
            min_x: level.west_boundary,
            min_y: level.south_boundary - 1.0,
            len_x: level.east_boundary - level.west_boundary,
            len_y: 1.0,
            platform_type: PlatformType::Line,
        });
        // East
        boundary_collision_lines.push(Platform {
            min_x: level.east_boundary,
            min_y: level.south_boundary,
            len_x: 1.0,
            len_y: level.north_boundary - level.south_boundary,
            platform_type: PlatformType::Line,
        });


        println!("{:#?}", blocks);

        Map {
            blocks: blocks,
            boundary_collision_lines: boundary_collision_lines,
            initial_x: level.initial_x,
            initial_y: level.initial_y,
            boundaries: [
                level.west_boundary,
                level.south_boundary,
                level.north_boundary - level.south_boundary,
                level.east_boundary - level.west_boundary,
            ],
        }
    }
}

impl From<level_serialization::Level> for Map {
    fn from(level: level_serialization::Level) -> Map {
        Self::from(&level)
    }
}
