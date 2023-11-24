mod art;
mod gif;

use std::{fs::File, io::Write};

use art::generate_art_level;
use gif::create_gif;
use i_wanna_build::{generic_level, serialize_level};

const IMAGE_PATH: &'static str = "";
const GIF_PATH: &'static str = "";
const IMAGE_LEVEL_PATH: &'static str = "";

fn main() {}

fn make_gif() {
    let level = create_gif(File::open("trollface-XL2x.gif").unwrap());
    let level = serialize_level(&level).unwrap();
    let mut file = File::create(GIF_PATH).unwrap();
    file.write_all(level.as_bytes()).unwrap();
}

fn make_art() {
    let image = image::open(IMAGE_PATH).unwrap();
    let level = generate_art_level(&image);
    let mut level_file = File::create(IMAGE_LEVEL_PATH).unwrap();
    let level = serialize_level(&level).unwrap();
    level_file.write_all(level.as_bytes()).unwrap();
}
