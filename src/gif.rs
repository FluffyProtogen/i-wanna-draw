use i_wanna_build::{map::*, *};
use image::EncodableLayout;
use std::io::Read;

const COLOR_THRESHOLD: u8 = 170;
const OFFSCREEN_OFFSET: u32 = 800;
const BLACK_BG: &str = "5A02000006000000050000000000000000000000000000000000000000000000000000000000000000000000000000000000000058B72CF87A2AD23F00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000F03F00000000000000000000F0BF00000000000000000000000000000000000000000000F03F000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";

pub fn create_gif<R: Read>(gif_file: R) -> Level {
    let mut level = generic_level("generated gif");
    level.maps[0].head.colors = BLACK_BG.into();

    let objects = &mut level.maps[0].objects;

    let (total_duration, frames) = collect_frames(gif_file);
    let total_duration = total_duration / 2;
    let delays = generate_delays(&frames, total_duration);

    let width = frames[0].width;
    let height = frames[0].height;

    for y in 0..height {
        for x in 0..width {
            if let Some(pixel_frame) = &delays[y][x] {
                let pixel = generate_pixel(x as u32, y as u32, pixel_frame, total_duration);
                objects.push(pixel);
            }
        }
    }

    level
}

fn is_solid(pixel: [u8; 4]) -> bool {
    pixel[0] > COLOR_THRESHOLD && pixel[1] > COLOR_THRESHOLD && pixel[2] > COLOR_THRESHOLD
}

fn collect_frames<R: Read>(gif_file: R) -> (u16, Vec<Frame>) {
    let mut decoder = gif::DecodeOptions::new();
    decoder.set_color_output(gif::ColorOutput::RGBA);
    let mut decoder = decoder.read_info(gif_file).unwrap();

    let mut frames = vec![];
    let mut total_duration = 0;
    while let Some(frame) = decoder.read_next_frame().unwrap() {
        let pixels = frame
            .buffer
            .as_bytes()
            .chunks(frame.width as usize * 4)
            .map(|bytes| {
                bytes
                    .chunks(4)
                    .map(|chunk| chunk.try_into().unwrap())
                    .collect()
            })
            .collect();
        frames.push(Frame {
            pixels,
            width: frame.width as usize,
            height: frame.height as usize,
        });
        total_duration += frame.delay;
    }
    (total_duration, frames)
}

fn generate_delays(frames: &[Frame], total_duration: u16) -> Vec<Vec<Option<PixelFrameInfo>>> {
    let width = frames[0].width;
    let height = frames[0].height;
    let mut grid = vec![vec![None; width]; height];
    let mut previous_states = vec![vec![false; width]; height];

    let frame_step = total_duration / frames.len() as u16;
    for (index, frame) in frames.iter().enumerate() {
        let index = index as u16;
        for y in 0..height {
            for x in 0..width {
                if is_solid(frame.pixels[y][x]) {
                    let frame_info: &mut Option<PixelFrameInfo> = &mut grid[y][x];
                    if let Some(pixel_frame) = frame_info {
                        if previous_states[y][x] == false {
                            let delay = index * frame_step;
                            pixel_frame.delays.push(delay);

                            previous_states[y][x] = true;
                        }
                    } else {
                        if index == 0 {
                            *frame_info = Some(PixelFrameInfo {
                                enabled_first_frame: true,
                                delays: vec![],
                            });
                        } else {
                            *frame_info = Some(PixelFrameInfo {
                                enabled_first_frame: false,
                                delays: vec![index * frame_step],
                            });
                        }
                        previous_states[y][x] = true;
                    }
                } else if let Some(pixel_frame) = &mut grid[y][x] {
                    if previous_states[y][x] == true {
                        let delay = index * frame_step;
                        pixel_frame.delays.push(delay);
                        previous_states[y][x] = false;
                    }
                }
            }
        }
    }

    grid
}

#[derive(Clone)]
struct PixelFrameInfo {
    enabled_first_frame: bool,
    delays: Vec<u16>,
}

struct Frame {
    pixels: Vec<Vec<[u8; 4]>>,
    width: usize,
    height: usize,
}

fn generate_pixel(x: u32, y: u32, pixel_frame: &PixelFrameInfo, total_duration: u16) -> Object {
    let x = x * 4;
    let y = y * 4;

    let x_offset = if pixel_frame.enabled_first_frame {
        0
    } else {
        OFFSCREEN_OFFSET
    };

    Object {
        type_id: 24,
        x: x + x_offset,
        y,
        rotation: None,
        params: vec![
            Param::new("blend_color", "0"),
            Param::new("scale", "0.2"),
            Param::new("layer", "4"),
            Param::new("tileset", "0"),
        ],
        nested_object: None,
        slot: None,
        events: generate_events(x, y, pixel_frame, total_duration),
    }
}

fn generate_events(
    x: u32,
    y: u32,
    pixel_frame: &PixelFrameInfo,
    total_duration: u16,
) -> Vec<Event> {
    let mut events = vec![];
    let event = {
        let x = if pixel_frame.enabled_first_frame {
            x
        } else {
            x + OFFSCREEN_OFFSET
        };
        metronome(x, y, 0, total_duration)
    };
    events.push(event);

    let iter = pixel_frame.delays.iter().enumerate().map(|(index, delay)| {
        let offset = if pixel_frame.enabled_first_frame {
            0
        } else {
            1
        };
        let x = if (index + offset) % 2 == 0 {
            x + OFFSCREEN_OFFSET
        } else {
            x
        };
        metronome(x, y, *delay, total_duration)
    });
    events.extend(iter);

    events
}

fn metronome(x: u32, y: u32, offset: u16, total_duration: u16) -> Event {
    Event {
        id: 17,
        params: vec![
            Param::new("offset", offset.to_string()),
            Param::new("frames", total_duration.to_string()),
        ],
        nested_events: vec![move_to(x, y)],
    }
}

fn move_to(x: u32, y: u32) -> Event {
    Event {
        id: 122,
        params: vec![
            Param::new("speed", "5000"),
            Param::new("position_y", y.to_string()),
            Param::new("set_position", "0"),
            Param::new("axis", "3"),
            Param::new("position_x", x.to_string()),
        ],
        nested_events: vec![],
    }
}
