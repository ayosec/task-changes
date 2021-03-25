//! Parser for `undo.data` file.

use std::cell::Cell;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

use nom::{
    branch::alt,
    bytes::complete::{tag, take_while, take_while1},
    character::complete::{char, digit1, newline},
    combinator::{all_consuming, map, map_res, recognize},
    multi::{many0, many1, separated_list0},
    sequence::{delimited, preceded, separated_pair, terminated},
    IResult,
};

#[derive(Debug, Default)]
pub struct Change {
    pub time: i64,
    pub old: Option<HashMap<String, String>>,
    pub new: HashMap<String, String>,
}

/// Parse `undo.data` files from Taskwarrior.
pub fn parse<T>(path: T) -> io::Result<Vec<Change>>
where
    T: AsRef<Path>,
{
    let path = path.as_ref();
    let contents = fs::read(path)?;

    let changes = match changes_list(&contents) {
        Ok((b"", changes)) => changes,
        _ => {
            eprintln!("Failed parse {}.", path.display());
            vec![]
        }
    };

    Ok(changes)
}

// Combinators.

type Error<'a> = nom::error::Error<&'a [u8]>;

fn space(i: &[u8]) -> IResult<&[u8], &[u8], Error> {
    take_while(|c| c == b' ' || c == b'\t')(i)
}

#[derive(Debug)]
enum ChangeFragment {
    Time(i64),
    Old(HashMap<String, String>),
    New(HashMap<String, String>),
}

fn integer(i: &[u8]) -> IResult<&[u8], i64, Error> {
    map_res(preceded(space, digit1), |i| {
        std::str::from_utf8(i).unwrap().parse()
    })(i)
}

fn quoted_string(i: &[u8]) -> IResult<&[u8], String, Error> {
    let escaped = Cell::new(false);
    let inner_value = move |c| {
        let was_escaped = escaped.replace(c == b'\\');
        c != b'"' || was_escaped
    };

    map_res(
        recognize(preceded(
            char('"'),
            terminated(take_while(inner_value), char('"')),
        )),
        serde_json::from_slice::<String>,
    )(i)
}

fn identifier(i: &[u8]) -> IResult<&[u8], &str, Error> {
    map_res(
        take_while1(|c: u8| c.is_ascii_alphanumeric() || c == b'_'),
        std::str::from_utf8,
    )(i)
}

fn attributes(i: &[u8]) -> IResult<&[u8], HashMap<String, String>, Error> {
    map(
        preceded(
            space,
            delimited(
                tag("["),
                separated_list0(
                    many1(char(' ')),
                    separated_pair(
                        preceded(space, identifier),
                        tag(":"),
                        preceded(space, quoted_string),
                    ),
                ),
                tag("]"),
            ),
        ),
        |items| {
            let mut map = HashMap::with_capacity(items.len());
            for (key, value) in items {
                map.insert(key.to_string(), value);
            }

            map
        },
    )(i)
}

fn fragment_time(i: &[u8]) -> IResult<&[u8], ChangeFragment, Error> {
    map(
        delimited(tag("time"), integer, preceded(space, newline)),
        ChangeFragment::Time,
    )(i)
}

fn fragment_old(i: &[u8]) -> IResult<&[u8], ChangeFragment, Error> {
    map(
        delimited(tag("old"), attributes, preceded(space, newline)),
        ChangeFragment::Old,
    )(i)
}

fn fragment_new(i: &[u8]) -> IResult<&[u8], ChangeFragment, Error> {
    map(
        delimited(tag("new"), attributes, preceded(space, newline)),
        ChangeFragment::New,
    )(i)
}

fn combine_fragments(fragments: Vec<ChangeFragment>) -> Change {
    let mut change = Change::default();
    for fragment in fragments {
        match fragment {
            ChangeFragment::Time(t) => change.time = t,
            ChangeFragment::Old(map) => change.old = Some(map),
            ChangeFragment::New(map) => change.new = map,
        }
    }

    change
}

fn change_item(i: &[u8]) -> IResult<&[u8], Change, Error> {
    map(
        delimited(
            space,
            many1(alt((fragment_time, fragment_old, fragment_new))),
            terminated(many1(char('-')), newline),
        ),
        combine_fragments,
    )(i)
}

fn changes_list(i: &[u8]) -> IResult<&[u8], Vec<Change>, Error> {
    all_consuming(many0(change_item))(i)
}
