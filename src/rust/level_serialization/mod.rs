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
    Box { x: f64, y: f64, width: f64, height: f64 },
    Line { x: f64, y: f64, direction: Direction, length: f64 }
}

#[derive(Debug, Clone)]
pub struct Level {
    pub initial_x: f64,
    pub initial_y: f64,
    pub items: Vec<LevelItem>,
}

named! {
    level_initial_coords<(f64, f64)>,

    map_res! (
        chain! (
            tag!("start")~
            opt!(call!(nom::space))~
            tag!(":")~
            opt!(call!(nom::space))~
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
            opt!(call!(nom::space))~
            tag!(":")~
            opt!(call!(nom::space))~
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
            opt!(call!(nom::space))~
            tag!(":")~
            opt!(call!(nom::space))~
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
        opt!(call!(nom::space))~
        call!(nom::eof),
        || ()
    )
}

named! {
    level<Level>,
    chain! (
        initial_coords: call!(level_initial_coords)~
        opt!(call!(nom::multispace))~
        items: terminated!(many0!(call!(level_item)), call!(level_end)),
        || {
            Level {
                initial_x: initial_coords.0,
                initial_y: initial_coords.1,
                items: items,
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


pub fn load_level<T: ?Sized>(input: &T) -> Option<Level> where T: AsRef<[u8]> {
    match level(input.as_ref()) {
        nom::IResult::Done(_, level) => Some(level),
        nom::IResult::Error(..) | nom::IResult::Incomplete(..) => None,
    }
}

#[allow(dead_code)]
pub fn save_level<T: ?Sized>(level: &Level, out: &mut T) -> io::Result<()> where T: Write {
    try!(write!(out, "start: {:.2},{:.2}\n\n", level.initial_x, level.initial_y));
    for item in &level.items {
        match item {
            &LevelItem::Box { x, y, width, height } => {
                try!(write!(out, "platform.box: {:.2},{:.2},{:.2},{:.2}\n", x, y, width, height));
            },
            &LevelItem::Line { x, y, direction, length } => {
                try!(write!(out, "platform.line: {:.2},{:.2},{},{:.2}\n", x, y, direction, length))
            }
        }
    }
    Ok(())
}
