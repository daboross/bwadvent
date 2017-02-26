use std::io::Write;
use std::convert::AsRef;
use std::str;
use std::io;
use std::fmt;

use nom;

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    North,
    East,
}

impl fmt::Display for Direction {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", match *self {
            Direction::North => "n",
            Direction::East => "e",
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum LevelItem {
    Box {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    },
    Line {
        x: f64,
        y: f64,
        direction: Direction,
        length: f64,
    },
}

#[derive(Debug, Clone)]
pub struct Level {
    pub initial_x: f64,
    pub initial_y: f64,
    pub items: Vec<LevelItem>,
    pub east_boundary: f64,
    pub south_boundary: f64,
    pub north_boundary: f64,
    pub west_boundary: f64,
}

named! {
    level_initial_coords<(f64, f64)>,

    map_res! (
        chain! (
            tag!("start")~
            opt!(complete!(call!(nom::space)))~
            tag!(":")~
            opt!(complete!(call!(nom::space)))~
            x: take_until_and_consume!(",")~
            y: take_until_and_consume!("\n"),
            || { (x, y) }
        ),
        |(x, y)| {
            Ok::<_, ()>((try!(parse_f64(x)), try!(parse_f64(y))))
        }
    )
}

named! {
    level_bounds<(f64, f64, f64, f64)>,

    map_res! (
        chain! (
            tag!("bounds")~
            opt!(complete!(call!(nom::space)))~
            tag!(":")~
            opt!(complete!(call!(nom::space)))~
            west: take_until_and_consume!(",")~
            south: take_until_and_consume!(",")~
            east: take_until_and_consume!(",")~
            north: take_until_and_consume!("\n"),
            || { (west, south, east, north) }
        ),
        |(west, south, east, north)| {
            Ok::<_, ()>((
                try!(parse_f64(west)),
                try!(parse_f64(south)),
                try!(parse_f64(east)),
                try!(parse_f64(north)),
            ))
        }
    )
}

named! {
    direction<Direction>,

    alt!(
        tag!("n") => { |_| Direction::North }
        | tag!("e") => { |_| Direction::East }
    )
}

named! {
    level_item_line<LevelItem>,

    map_res! (
        chain! (
            tag!("platform.line")~
            opt!(complete!(call!(nom::space)))~
            tag!(":")~
            opt!(complete!(call!(nom::space)))~
            x: take_until_and_consume!(",")~
            y: take_until_and_consume!(",")~
            d: call!(direction)~ tag!(",")~
            l: take_until_and_consume!("\n"),
            || (x, y, d, l)
        ),
        |(x, y, d, l)| {
            Ok::<_, ()>(LevelItem::Line {
                x: try!(parse_f64(x)),
                y: try!(parse_f64(y)),
                direction: d,
                length: try!(parse_f64(l)),
            })
        }
    )
}

named! {
    level_item_box<LevelItem>,

    map_res! (
        chain! (
            tag!("platform.box")~
            opt!(complete!(call!(nom::space)))~
            tag!(":")~
            opt!(complete!(call!(nom::space)))~
            x: take_until_and_consume!(",")~
            y: take_until_and_consume!(",")~
            w: take_until_and_consume!(",")~
            h: take_until_and_consume!("\n"),
            || (x, y, w, h)
        ),
        |(x, y, w, h)| {
            Ok::<_, ()>(LevelItem::Box {
                x: try!(parse_f64(x)),
                y: try!(parse_f64(y)),
                width: try!(parse_f64(w)),
                height: try!(parse_f64(h)),
            })
        }
    )
}

named! {
    level_item<LevelItem>,
    alt!(call!(level_item_box) | call!(level_item_line))
}

named! {
    level_end<()>,
    chain! (
        opt!(complete!(call!(nom::multispace)))~
        eof!(),
        || ()
    )
}

named! {
    level<Level>,
    chain! (
        initial_coords: call!(level_initial_coords)~
        opt!(complete!(call!(nom::multispace)))~
        bounds: call!(level_bounds)~
        opt!(complete!(call!(nom::multispace)))~
        items: terminated!(many0!(call!(level_item)), call!(level_end)),
        || {
            Level {
                initial_x: initial_coords.0,
                initial_y: initial_coords.1,
                items: items,
                west_boundary: bounds.0,
                south_boundary: bounds.1,
                east_boundary: bounds.2,
                north_boundary: bounds.3,
            }
        }
    )
}

fn parse_f64(i: &[u8]) -> Result<f64, ()> {
    match str::from_utf8(i) {
        Ok(v) => match v.parse::<f64>() {
            Ok(v) => Ok(v),
            Err(_) => Err(()),
        },
        Err(_) => Err(()),
    }
}


pub fn load_level<T: ?Sized>(input: &T) -> Result<Level, nom::IError> where T: AsRef<[u8]> {
    level(input.as_ref()).to_full_result()
}

#[allow(dead_code)]
pub fn save_level<T: ?Sized>(level: &Level, out: &mut T) -> io::Result<()> where T: Write {
    try!(write!(out, "start: {:.2},{:.2}\n\n", level.initial_x, level.initial_y));
    try!(write!(out, "bounds: {:.2},{:.2},{:.2},{:.2}\n\n",
        level.east_boundary, level.south_boundary, level.west_boundary, level.north_boundary));
    for item in &level.items {
        match *item {
            LevelItem::Box { x, y, width, height } => {
                try!(write!(out, "platform.box: {:.2},{:.2},{:.2},{:.2}\n", x, y, width, height));
            }
            LevelItem::Line { x, y, direction, length } => {
                try!(write!(out, "platform.line: {:.2},{:.2},{},{:.2}\n", x, y, direction, length))
            }
        }
    }
    Ok(())
}
