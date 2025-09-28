use image::{ColorType, DynamicImage, ImageBuffer, imageops};
use imageproc::drawing::{Canvas, draw_filled_rect_mut, draw_line_segment_mut};
use imageproc::rect::Rect;
use std::path::Path;

use crate::ast::DepthExpr;

type Offset = (u32, u32); // x, y or width, height

const COMPONENT_WIDTH: u32 = 20;
const COMPONENT_HEIGHT: u32 = 20;

const DIRECTION_LINE_THICKNESS: u32 = 2;
const DIRECTION_LINE_LENGTH: u32 = 20;

const RESOLUTION_MULTIPLIER: u32 = 50;

/*
offset based mutation model, to avoid overflowing on previous image
*/

mod colors {
    use image::Rgba;
    pub const STARSTOP: Rgba<u8> = Rgba([
        63, 122, 86, 255,
    ]);
    pub const PROCESS: Rgba<u8> = Rgba([
        0, 104, 124, 255,
    ]);
    pub const IO: Rgba<u8> = Rgba([
        149, 30, 88, 255,
    ]);
    pub const DIRECT: Rgba<u8> = Rgba([
        31, 133, 53, 255,
    ]);
    pub const BG: Rgba<u8> = Rgba([
        0, 0, 0, 0,
    ]);
}

const fn res(original: u32) -> u32 {
    original * RESOLUTION_MULTIPLIER
}

fn ext(img: &mut DynamicImage, (curw, curh): &mut Offset, (extw, exth): &mut Offset) {
    let (w, h) = img.dimensions();
    let extedw = *extw + *curw;
    let extedh = *exth + *curh;
    if extedw <= w && extedh <= h {
        return;
    }
    let mut resized = ImageBuffer::from_pixel(w.max(extedw), h.max(extedh), colors::BG);
    imageops::overlay(&mut resized, img, 0, 0);
    *img = image::DynamicImage::ImageRgba8(resized);
}

fn draw_process(img: &mut DynamicImage, (curw, curh): &mut Offset) -> Offset {
    *curw = curw
        .checked_sub(res(COMPONENT_WIDTH) / 2)
        .unwrap_or(0);
    ext(img, &mut (*curw, *curh), &mut (res(COMPONENT_WIDTH), res(COMPONENT_HEIGHT)));
    draw_filled_rect_mut(img, Rect::at(*curw as i32, *curh as i32).of_size(res(COMPONENT_WIDTH), res(COMPONENT_HEIGHT)), colors::PROCESS);
    *curw += res(COMPONENT_WIDTH) / 2;
    *curh += res(COMPONENT_HEIGHT);
    (*curw, *curh)
}

fn draw_direction(img: &mut DynamicImage, (oriw, orih): &mut Offset) -> Offset {
    *oriw = oriw
        .checked_sub(res(DIRECTION_LINE_THICKNESS) / 2)
        .unwrap_or(0);
    ext(img, &mut (*oriw, *orih), &mut (res(DIRECTION_LINE_THICKNESS), res(DIRECTION_LINE_LENGTH)));
    draw_filled_rect_mut(img, Rect::at(*oriw as i32, *orih as i32).of_size(res(DIRECTION_LINE_THICKNESS), res(DIRECTION_LINE_LENGTH)), colors::DIRECT);
    *oriw += res(DIRECTION_LINE_THICKNESS) / 2;
    *orih += res(DIRECTION_LINE_LENGTH);
    (*oriw, *orih)
}

fn build(ast: &[DepthExpr]) -> DynamicImage {
    let mut img = DynamicImage::new(0, 0, ColorType::Rgba8);
    let mut offset = (0, 0);
    draw_process(&mut img, &mut offset);
    draw_direction(&mut img, &mut offset);
    draw_process(&mut img, &mut offset);

    img
}

pub fn create(ast: &[DepthExpr], path: &Path) {
    build(ast)
        .save(path)
        .unwrap();
}
