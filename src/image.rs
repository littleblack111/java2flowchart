use ab_glyph::{FontArc, PxScale};
use fontdb::{Database, Family, Query, Source};
use image::{ColorType, DynamicImage, ImageBuffer, imageops};
use imageproc::drawing::{Canvas, draw_filled_rect_mut, draw_polygon_mut, draw_text_mut, text_size};
use imageproc::point::Point;
use imageproc::rect::Rect;
use std::path::Path;
use std::{fs, io};

use crate::ast::DepthExpr;

type Offset = (u32, u32, u32); // x, y or width, height center
type NCOffset = (u32, u32);

// TODO: based on how much text
const COMPONENT_TEXT_PADDING: u32 = 1 * RESOLUTION_MULTIPLIER;

const TEXT_SCALE: f32 = 12.0 * RESOLUTION_MULTIPLIER as f32;
const TEXT_LEN_WRAP: usize = 10;

// TODO: LENGTH based on where to where
const DIRECTION_LINE_THICKNESS: u32 = 2 * RESOLUTION_MULTIPLIER / 2;
const DIRECTION_LINE_LENGTH: u32 = 13 * RESOLUTION_MULTIPLIER;
const DIRECTION_LINE_ARROW_OFFSET: u32 = 5 * RESOLUTION_MULTIPLIER / 2;

const RESOLUTION_MULTIPLIER: u32 = 5;

/*
offset based mutation model, to avoid overflowing on previous image
*/

#[derive(PartialEq)]
enum HDirection {
    Up,
    Down,
}

enum VDirection {
    Left,
    Right,
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

    match &db
        .face(
            db.query(&Query {
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
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no system fonts"))?,
        )
        .ok_or_else(|| io::Error::other("face missing"))?
        .source
    {
        Source::File(path) => fs::read(path).and_then(|bytes| FontArc::try_from_vec(bytes).map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "parse font failed"))),
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
fn draw_process(img: &mut DynamicImage, txt: &str, (curw, curh, c): &mut Offset, dir: HDirection) {
    let wrapped = wrap_str(txt);

    let pxscale = PxScale::from(TEXT_SCALE / 2_f32);
    let font = &get_font().unwrap();
    let (text_w, _) = text_size(pxscale, font, wrapped[0].as_str());
    let w = text_w + (2 * COMPONENT_TEXT_PADDING);
    let h = TEXT_SCALE as u32 / 2 * wrapped.len() as u32 + (2 * COMPONENT_TEXT_PADDING); // text_size's height is weird and incorrect
    if dir == HDirection::Up {
        *curh -= h;
    }
    ext(img, &mut (*c, *curh), &mut (w, h));
    let cw = c
        .checked_sub(w / 2)
        .unwrap_or(0);
    draw_filled_rect_mut(img, Rect::at(cw as i32, *curh as i32).of_size(w, h), colors::PROCESS);
    // TODO: move to draw_text()
    for s in wrapped {
        draw_text_mut(img, colors::FG, cw as i32 + COMPONENT_TEXT_PADDING as i32, *curh as i32 + COMPONENT_TEXT_PADDING as i32, pxscale, font, s.as_str());
        *curh += TEXT_SCALE as u32 / 2;
    }
    if *c == 0 {
        *c = *curw + (w / 2);
    }
    *curh += 2 * COMPONENT_TEXT_PADDING;
}

fn draw_direction(img: &mut DynamicImage, (_, orih, c): &mut Offset, dst: Option<&NCOffset>) {
    let srcx = *c as i32;
    let srcy = *orih as i32;

    // perpendicular(thickness)
    let (dstx, dsty) = match dst {
        Some(&(x, y)) => {
            let dx = if x == 0 {
                srcx
            } else {
                x as i32
            };
            (dx, y as i32)
        }
        None => (srcx, srcy + (DIRECTION_LINE_LENGTH - DIRECTION_LINE_ARROW_OFFSET) as i32),
    };

    let xdiff = (dstx - srcx) as f32;
    let ydiff = (dsty - srcy) as f32;
    let length = (xdiff * xdiff + ydiff * ydiff)
        .sqrt()
        .max(1_f32);

    let prepx = -ydiff / length;
    let prepy = xdiff / length;

    let linemaxx = (dstx as f32 - (xdiff / length) * DIRECTION_LINE_ARROW_OFFSET as f32).round() as i32;
    let linemaxy = (dsty as f32 - (ydiff / length) * DIRECTION_LINE_ARROW_OFFSET as f32).round() as i32;

    let line = [
        Point::new((srcx as f32 - prepx * DIRECTION_LINE_THICKNESS as f32).round() as i32, (srcy as f32 - prepy * DIRECTION_LINE_THICKNESS as f32).round() as i32),
        Point::new((srcx as f32 + prepx * DIRECTION_LINE_THICKNESS as f32).round() as i32, (srcy as f32 + prepy * DIRECTION_LINE_THICKNESS as f32).round() as i32),
        Point::new((linemaxx as f32 + prepx * DIRECTION_LINE_THICKNESS as f32).round() as i32, (linemaxy as f32 + prepy * DIRECTION_LINE_THICKNESS as f32).round() as i32),
        Point::new((linemaxx as f32 - prepx * DIRECTION_LINE_THICKNESS as f32).round() as i32, (linemaxy as f32 - prepy * DIRECTION_LINE_THICKNESS as f32).round() as i32),
    ];

    let xc = dstx as f32 - (xdiff / length) * DIRECTION_LINE_ARROW_OFFSET as f32;
    let yc = dsty as f32 - (ydiff / length) * DIRECTION_LINE_ARROW_OFFSET as f32;

    let left = Point::new((xc + prepx * DIRECTION_LINE_ARROW_OFFSET as f32).round() as i32, (yc + prepy * DIRECTION_LINE_ARROW_OFFSET as f32).round() as i32);
    let right = Point::new((xc - prepx * DIRECTION_LINE_ARROW_OFFSET as f32).round() as i32, (yc - prepy * DIRECTION_LINE_ARROW_OFFSET as f32).round() as i32);
    let tip = Point::new(dstx, dsty);
    let center = Point::new(xc.round() as i32, yc.round() as i32);

    let mut maxx = srcx.max(dstx);
    let mut maxy = srcy.max(dsty);
    for p in line
        .iter()
        .copied()
        .chain([
            tip, center, left, right,
        ])
    {
        if p.x > maxx {
            maxx = p.x;
        }
        if p.y > maxy {
            maxy = p.y;
        }
    }
    ext(img, &mut (*c, *orih), &mut (((maxx - *c as i32).max(0) as u32), ((maxy - *orih as i32).max(0) as u32)));

    draw_polygon_mut(img, &line, colors::DIRECT);
    draw_polygon_mut(
        img,
        &[
            tip, left, center,
        ],
        colors::DIRECT,
    );
    draw_polygon_mut(
        img,
        &[
            tip, right, center,
        ],
        colors::DIRECT,
    );

    *c = dstx as u32;
    *orih = dsty as u32;
}

fn build(ast: &[DepthExpr]) -> DynamicImage {
    let mut img = DynamicImage::new(0, 0, ColorType::Rgba8);
    // TODO: move to mutable singleton
    let mut offset: Offset = (0, 0, 0);
    draw_process(&mut img, "abcdefghijklmnop", &mut offset, HDirection::Down);
    draw_direction(&mut img, &mut offset, None);
    draw_process(&mut img, "a", &mut offset, HDirection::Down);
    draw_direction(&mut img, &mut offset, None);
    draw_process(&mut img, "a", &mut offset, HDirection::Down);
    let (a, b, c) = offset.clone();
    draw_direction(&mut img, &mut offset, None);
    draw_process(&mut img, "a", &mut offset, HDirection::Down);
    draw_direction(&mut img, &mut offset, None);
    draw_process(&mut img, "a", &mut offset, HDirection::Down);
    draw_direction(&mut img, &mut offset, Some(&(a + 100, b)));
    draw_process(&mut img, "a", &mut offset, HDirection::Up);

    img
}

pub fn create(ast: &[DepthExpr], path: &Path) {
    build(ast)
        .save(path)
        .unwrap();
}
