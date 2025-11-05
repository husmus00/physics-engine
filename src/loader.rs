use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;
use std::error::Error;

use crate::things::*;

#[derive(Debug, Serialize, Deserialize)]
enum Sprite {
    Circle,
    Rectangle,
}

#[derive(Debug, Serialize, Deserialize)]
struct ThingData {
    sprite: Sprite,
    x_pos: u16,
    y_pos: u16,
    width: u16,
    height: u16,
    rotation: u16,
    dynamic: bool,
    r: u8,
    g: u8,
    b: u8,
}

fn load_things_from_file(path: &str) -> Result<Vec<Thing>, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let thing_data: Vec<ThingData> = serde_json::from_reader(reader)?;

    for data in thing_data {

    }

    Ok(sprites)
}

fn main() -> Result<(), Box<dyn Error>> {
    let sprites = load_things_from_file("sprites.json")?;

    for sprite in sprites {
        println!("{:?}", sprite);
    }

    Ok(())
}
