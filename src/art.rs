use i_wanna_build::{map::*, *};
use image::{DynamicImage, GenericImageView};

pub fn generate_art_level(image: &DynamicImage) -> Level {
    let mut level = generic_level("generated image");
    let objects = &mut level.maps[0].objects;
    for y in 0..image.height() {
        for x in 0..image.width() {
            let pixel = image.get_pixel(x, y).0;
            if pixel[3] < 10 {
                continue;
            }

            let pixel = generate_pixel(x, y, pixel);
            objects.push(pixel);
        }
    }

    level
}

fn pixel_color_to_blend_color(pixel: [u8; 4]) -> u32 {
    (pixel[0] as u32) + ((pixel[1] as u32) << 8) + ((pixel[2] as u32) << 16)
}

fn generate_pixel(x: u32, y: u32, pixel: [u8; 4]) -> Object {
    Object {
        type_id: 24,
        x: x * 4,
        y: y * 4,
        rotation: None,
        params: vec![
            Param::new("blend_color", pixel_color_to_blend_color(pixel).to_string()),
            Param::new("scale", "0.2"),
            Param::new("layer", "4"),
            Param::new("tileset", "0"),
        ],
        nested_object: None,
        slot: None,
        events: vec![],
    }
}
