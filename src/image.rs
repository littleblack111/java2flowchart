use ab_glyph::{FontArc, PxScale};
use fontdb::{Database, Family, Query, Source};
use image::{ColorType, DynamicImage, ImageBuffer, imageops};
use imageproc::drawing::{Canvas, draw_filled_rect_mut, draw_polygon_mut, draw_text_mut};
use imageproc::point::Point;
use imageproc::rect::Rect;
use std::io;
use std::path::Path;

use crate::ast::DepthExpr;

type Offset = (u32, u32, u32); // x, y or width, height center
type NCOffset = (u32, u32);

// TODO: based on how much text
const COMPONENT_TEXT_PADDING: u32 = 1 * RESOLUTION_MULTIPLIER;

const TEXT_SCALE: f32 = 12.0 * RESOLUTION_MULTIPLIER as f32;
const TEXT_LEN_WRAP: usize = 10;

// TODO: LENGTH based on where to where
const DIRECTION_LINE_THICKNESS: u32 = 2 * RESOLUTION_MULTIPLIER;
const DIRECTION_LINE_LENGTH: u32 = 13 * RESOLUTION_MULTIPLIER;
const DIRECTION_LINE_ARROW_OFFSET: u32 = 5 * RESOLUTION_MULTIPLIER / 2;

const RESOLUTION_MULTIPLIER: u32 = 50;

/*
offset based mutation model, to avoid overflowing on previous image
*/

enum Direction {
    UP,
    DOWN,
    LEFT,
    RIGHT,
}

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
    pub const FG: Rgba<u8> = Rgba([
        255, 255, 255, 255,
    ]);
}

fn wrap_str(s: &str) -> Vec<String> {
    s.chars()
        .collect::<Vec<_>>()
        .chunks(TEXT_LEN_WRAP)
        .map(|c| {
            c.iter()
                .collect()
        })
        .collect()
}

// TODO: store as singleton
fn get_font() -> Result<FontArc, io::Error> {
    let mut db = Database::new();
    db.load_system_fonts();

    let id = db
        .query(&Query {
            families: &[Family::SansSerif],
            weight: fontdb::Weight::NORMAL,
            stretch: fontdb::Stretch::Normal,
            style: fontdb::Style::Normal,
        })
        .or_else(|| {
            db.faces()
                .next()
                .map(|f| f.id)
        })
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no system fonts"))?;

    let face = db
        .face(id)
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "face missing"))?;

    match &face.source {
        Source::File(path) => std::fs::read(path).and_then(|bytes| FontArc::try_from_vec(bytes).map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "parse font failed"))),
        Source::Binary(data) => FontArc::try_from_vec(
            data.as_ref()
                .as_ref()
                .to_vec(),
        )
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "parse font failed")),
        _ => Err(io::Error::new(io::ErrorKind::Unsupported, "unsupported font source")),
    }
}

fn ext(img: &mut DynamicImage, (curw, curh): &mut NCOffset, (extw, exth): &mut NCOffset) {
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

// TODO: make draw_rect func
fn draw_process(img: &mut DynamicImage, txt: &str, (curw, curh, c): &mut Offset) {
    let wrapped = wrap_str(txt);
    let w = (TEXT_SCALE as u32 / 5
        * if wrapped.len() <= 1 {
            txt.len() as u32
        } else {
            TEXT_LEN_WRAP as u32
        })
        + (2 * COMPONENT_TEXT_PADDING);
    let h = TEXT_SCALE as u32 / 2 * wrapped.len() as u32 + (2 * COMPONENT_TEXT_PADDING);
    ext(img, &mut (*c, *curh), &mut (w, h));
    let cw = c
        .checked_sub(w / 2)
        .unwrap_or(0);
    draw_filled_rect_mut(img, Rect::at(cw as i32, *curh as i32).of_size(w, h), colors::PROCESS);
    // TODO: move to draw_text()
    for s in wrapped {
        draw_text_mut(img, colors::FG, cw as i32 + COMPONENT_TEXT_PADDING as i32, *curh as i32 + COMPONENT_TEXT_PADDING as i32, PxScale::from(TEXT_SCALE / 2_f32), &get_font().unwrap(), s.as_str());
        *curh += TEXT_SCALE as u32 / 2;
    }
    *c = *curw + (w / 2);
    *curh += 2 * COMPONENT_TEXT_PADDING;
}

fn draw_direction(img: &mut DynamicImage, (oriw, orih, c): &mut Offset) {
    ext(img, &mut (*c, *orih), &mut (DIRECTION_LINE_THICKNESS, DIRECTION_LINE_LENGTH));
    draw_filled_rect_mut(img, Rect::at(*c as i32 - (DIRECTION_LINE_THICKNESS as i32 / 2), *orih as i32).of_size(DIRECTION_LINE_THICKNESS, DIRECTION_LINE_LENGTH - DIRECTION_LINE_ARROW_OFFSET), colors::DIRECT);
    *orih += DIRECTION_LINE_LENGTH;

    // arrow
    let x = *c as i32;
    let y = *orih as i32;
    let offset = DIRECTION_LINE_ARROW_OFFSET as i32;
    // left
    draw_polygon_mut(
        img,
        &[
            Point::new(x, y),
            Point::new(x - offset, y - offset),
            Point::new(x, y - offset),
        ],
        colors::DIRECT,
    );
    // right
    draw_polygon_mut(
        img,
        &[
            Point::new(x, y),
            Point::new(x + offset, y - offset),
            Point::new(x, y - offset),
        ],
        colors::DIRECT,
    );
}

fn build(ast: &[DepthExpr]) -> DynamicImage {
    let mut img = DynamicImage::new(0, 0, ColorType::Rgba8);
    // TODO: move to mutable singleton
    let mut offset: Offset = (0, 0, 0);
    draw_process(&mut img, "abcdefghijklmnop", &mut offset);
    draw_direction(&mut img, &mut offset);
    draw_process(&mut img, "a", &mut offset);

    img
}

pub fn create(ast: &[DepthExpr], path: &Path) {
    build(ast)
        .save(path)
        .unwrap();
}
