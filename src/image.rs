use image::{ColorType, DynamicImage, GenericImage, ImageBuffer, imageops};
use imageproc::drawing::{Canvas, draw_cross_mut, draw_filled_circle_mut, draw_filled_rect_mut, draw_hollow_circle_mut, draw_hollow_rect_mut, draw_line_segment_mut};
use imageproc::rect::Rect;
use std::path::Path;

use crate::ast::DepthExpr;

type Offset = (u32, u32); // x, y or width, height

const COMPONENT_WIDTH: u32 = 20;
const COMPONENT_HEIGHT: u32 = 20;

const DIRECTION_LINE_HEIGHT: u32 = 100;
const DIRECTION_LINE_WIDTH: u32 = 20;

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

fn ext(img: &mut DynamicImage, (curw, curh): &mut Offset, (extw, exth): &mut Offset) {
    let (w, h) = img.dimensions();
    let extedw = *extw + *curw;
    let extedh = *exth + *curh;
    if extedw < w || extedh < h {
        return;
    }
    let mut resized = ImageBuffer::from_pixel(extedw, extedh, colors::BG);
    imageops::overlay(&mut resized, img, 0, 0);
    *img = image::DynamicImage::ImageRgba8(resized);
}

fn draw_process(img: &mut DynamicImage, (curw, curh): &mut Offset) -> Offset {
    ext(img, &mut (*curw, *curh), &mut (COMPONENT_WIDTH, COMPONENT_HEIGHT));
    draw_filled_rect_mut(img, Rect::at(*curw as i32, *curh as i32).of_size(COMPONENT_WIDTH, COMPONENT_HEIGHT), colors::PROCESS);
    *curw = COMPONENT_WIDTH / 2;
    *curh += COMPONENT_HEIGHT;
    (*curw, *curh)
}

fn draw_direction(img: &mut DynamicImage, (oriw, orih): &mut Offset) -> Offset {
    ext(img, &mut (*oriw, *orih), &mut (DIRECTION_LINE_WIDTH, DIRECTION_LINE_HEIGHT));
    draw_line_segment_mut(img, (*oriw as f32, *orih as f32), ((*oriw + DIRECTION_LINE_WIDTH) as f32, (*orih + DIRECTION_LINE_HEIGHT) as f32), colors::DIRECT);
    *orih += DIRECTION_LINE_HEIGHT;
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
