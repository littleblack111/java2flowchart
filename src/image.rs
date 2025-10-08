use ab_glyph::{FontArc, PxScale};
use fontdb::{Database, Family, Query, Source};
use image::{ColorType, DynamicImage, ImageBuffer, imageops};
use imageproc::{
    drawing::{Canvas, draw_filled_rect_mut, draw_polygon_mut, draw_text_mut, text_size},
    point::Point,
    rect::Rect,
};
use std::{fs, io, path::Path};

use crate::ast::DepthExpr;

/*
offset based mutation model, to avoid overflowing on previous image
*/

pub struct FlowChart {
    img: DynamicImage,
    offset: Offset,
    font: FontArc,
}

// TODO: change to i32, since we converted it to i32 already so the extra len of
// u32 is useless
#[derive(Clone, Copy)]
struct Offset {
    x: u32,
    y: u32,
    center: u32,
}

#[derive(Clone, Copy)]
struct NCOffset {
    x: u32,
    y: u32,
}

struct MutNCOffset<'a> {
    x: &'a mut u32,
    y: &'a mut u32,
}

impl Default for FlowChart {
    fn default() -> Self {
        Self::new()
    }
}

impl FlowChart {
    // TODO: based on how much text
    const COMPONENT_TEXT_PADDING: u32 = 1 * Self::RESOLUTION_MULTIPLIER;

    const TEXT_SCALE: f32 = 12.0 * Self::RESOLUTION_MULTIPLIER as f32;
    const TEXT_LEN_WRAP: usize = 10;

    // TODO: LENGTH based on where to where
    const DIRECTION_LINE_THICKNESS: u32 = 2 * Self::RESOLUTION_MULTIPLIER / 2;
    const DIRECTION_LINE_LENGTH: u32 = 13 * Self::RESOLUTION_MULTIPLIER;
    const DIRECTION_LINE_ARROW_OFFSET: u32 = 5 * Self::RESOLUTION_MULTIPLIER / 2;

    const RESOLUTION_MULTIPLIER: u32 = 5;

    pub fn new() -> Self {
        Self {
            img: DynamicImage::new(0, 0, ColorType::Rgba8),
            offset: Offset {
                x: 0,
                y: 0,
                center: 0,
            },
            font: Self::get_system_font(),
        }
    }

    pub fn create(ast: &[DepthExpr], path: &Path) {
        let mut chart = FlowChart::new();
        chart.build(ast);
        chart
            .img
            .save(path)
            .unwrap();
    }

    fn get_system_font() -> FontArc {
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
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no system fonts"))
                .expect("no system fonts"),
            )
            .ok_or_else(|| io::Error::other("face missing"))
            .expect("face missing")
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
        .expect("failed to load font")
    }
    fn wrap_str(s: &str) -> Vec<String> {
        s.chars()
            .collect::<Vec<_>>()
            .chunks(Self::TEXT_LEN_WRAP)
            .map(|c| {
                c.iter()
                    .collect()
            })
            .collect()
    }

    fn ext(&mut self, offset: &mut NCOffset, ext: &mut NCOffset) {
        let (curw, curh) = &mut (offset.x, offset.y);
        let (extw, exth) = &mut (ext.x, ext.y);
        let img = &mut self.img;
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

    fn draw_text(&mut self, txts: &[&str], pxscale: &PxScale, offset: &mut MutNCOffset) {
        let (curw, curh) = (&offset.x, &mut offset.y);
        for txt in txts {
            draw_text_mut(&mut self.img, colors::FG, **curw as i32 + Self::COMPONENT_TEXT_PADDING as i32, **curh as i32 + Self::COMPONENT_TEXT_PADDING as i32, *pxscale, &self.font, txt);
            **curh += Self::TEXT_SCALE as u32 / 2;
        }
    }

    // TODO: make draw_rect func
    // TODO: Accept all directions
    fn draw_process(&mut self, txt: &str, config: ComponentConfig<HDirection, HDirection>) {
        let curh = &mut self
            .offset
            .y;

        let wrapped = Self::wrap_str(txt);
        let wrapped: Vec<&str> = wrapped
            .iter()
            .map(|s| s.as_str())
            .collect();

        let pxscale = PxScale::from(Self::TEXT_SCALE / 2_f32);
        let (text_w, _) = text_size(pxscale, &self.font, wrapped[0]);
        let w = text_w + (2 * Self::COMPONENT_TEXT_PADDING);
        let h = Self::TEXT_SCALE as u32 / 2 * wrapped.len() as u32 + (2 * Self::COMPONENT_TEXT_PADDING); // text_size's height is weird and incorrect
        if config.dst_direction == HDirection::Up {
            *curh = curh
                .checked_sub(h)
                .unwrap_or(0);
        }
        let (curh, c) = (
            self.offset
                .y,
            self.offset
                .center,
        );
        self.ext(
            &mut NCOffset {
                x: c,
                y: curh,
            },
            &mut NCOffset {
                x: w,
                y: h,
            },
        );
        let (curw, mut curh, c) = (
            self.offset
                .x,
            self.offset
                .y,
            self.offset
                .center,
        );
        let mut cw = c.saturating_sub(w / 2);
        draw_filled_rect_mut(&mut self.img, Rect::at(cw as i32, curh as i32).of_size(w, h), colors::PROCESS);
        self.draw_text(
            &wrapped,
            &pxscale,
            // FIXME: i don't think this works, we giving them derefed value
            &mut MutNCOffset {
                x: &mut cw,
                y: &mut curh,
            },
        );
        (
            self.offset
                .x,
            self.offset
                .y,
            self.offset
                .center,
        ) = (curw, curh, c);
        if self
            .offset
            .center
            == 0
        {
            self.offset
                .center = curw + (w / 2);
        }
        self.offset
            .y += 2 * Self::COMPONENT_TEXT_PADDING;
    }

    fn draw_direction(&mut self, src: Option<&NCOffset>, dst: Option<&NCOffset>) {
        let (orih, c) = (
            self.offset
                .y,
            self.offset
                .center,
        );

        let (srcx, srcy) = match src {
            Some(&offset) => {
                let dx = if offset.x == 0 {
                    c as i32
                } else {
                    offset.x as i32
                };
                (dx, offset.y as i32)
            }
            None => (c as i32, orih as i32),
        };

        // perpendicular(thickness)
        let (dstx, dsty) = match dst {
            Some(&offset) => {
                let dx = if offset.x == 0 {
                    srcx
                } else {
                    offset.x as i32
                };
                (dx, offset.y as i32)
            }
            None => (srcx, srcy + (Self::DIRECTION_LINE_LENGTH - Self::DIRECTION_LINE_ARROW_OFFSET) as i32),
        };

        let xdiff = (dstx - srcx) as f32;
        let ydiff = (dsty - srcy) as f32;
        let length = (xdiff * xdiff + ydiff * ydiff)
            .sqrt()
            .max(1_f32);

        let prepx = -ydiff / length;
        let prepy = xdiff / length;

        let linemaxx = (dstx as f32 - (xdiff / length) * Self::DIRECTION_LINE_ARROW_OFFSET as f32).round() as i32;
        let linemaxy = (dsty as f32 - (ydiff / length) * Self::DIRECTION_LINE_ARROW_OFFSET as f32).round() as i32;

        let line = [
            Point::new((srcx as f32 - prepx * Self::DIRECTION_LINE_THICKNESS as f32).round() as i32, (srcy as f32 - prepy * Self::DIRECTION_LINE_THICKNESS as f32).round() as i32),
            Point::new((srcx as f32 + prepx * Self::DIRECTION_LINE_THICKNESS as f32).round() as i32, (srcy as f32 + prepy * Self::DIRECTION_LINE_THICKNESS as f32).round() as i32),
            Point::new((linemaxx as f32 + prepx * Self::DIRECTION_LINE_THICKNESS as f32).round() as i32, (linemaxy as f32 + prepy * Self::DIRECTION_LINE_THICKNESS as f32).round() as i32),
            Point::new((linemaxx as f32 - prepx * Self::DIRECTION_LINE_THICKNESS as f32).round() as i32, (linemaxy as f32 - prepy * Self::DIRECTION_LINE_THICKNESS as f32).round() as i32),
        ];

        let xc = dstx as f32 - (xdiff / length) * Self::DIRECTION_LINE_ARROW_OFFSET as f32;
        let yc = dsty as f32 - (ydiff / length) * Self::DIRECTION_LINE_ARROW_OFFSET as f32;

        let left = Point::new((xc + prepx * Self::DIRECTION_LINE_ARROW_OFFSET as f32).round() as i32, (yc + prepy * Self::DIRECTION_LINE_ARROW_OFFSET as f32).round() as i32);
        let right = Point::new((xc - prepx * Self::DIRECTION_LINE_ARROW_OFFSET as f32).round() as i32, (yc - prepy * Self::DIRECTION_LINE_ARROW_OFFSET as f32).round() as i32);
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
        self.ext(
            &mut NCOffset {
                x: c,
                y: orih,
            },
            &mut NCOffset {
                x: ((maxx - c as i32).max(0) as u32),
                y: ((maxy - orih as i32).max(0) as u32),
            },
        );
        let img = &mut self.img;

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

        self.offset
            .center = dstx as u32;
        self.offset
            .y = dsty as u32;
    }

    fn build(&mut self, ast: &[DepthExpr]) {
        // for node in ast {
        //     match node {
        //         DepthExpr::Decision {
        //             cond,
        //             t,
        //             then_branch,
        //             else_branch,
        //         } => todo!(),
        //         DepthExpr::IO(_) => todo!(),
        //         DepthExpr::Process(_) => todo!(),
        //     }
        // }
        self.draw_process(
            "abcdefghijklmnospa",
            ComponentConfig {
                ori_direction: HDirection::Down,
                dst_direction: HDirection::Down,
            },
        );
        self.draw_direction(None, None);
        self.draw_process(
            "abc",
            ComponentConfig {
                ori_direction: HDirection::Down,
                dst_direction: HDirection::Down,
            },
        );
        self.draw_direction(None, None);
        self.draw_process(
            "a",
            ComponentConfig {
                ori_direction: HDirection::Down,
                dst_direction: HDirection::Down,
            },
        );
        let offset = self.offset;
        self.draw_direction(None, None);
        self.draw_process(
            "a",
            ComponentConfig {
                ori_direction: HDirection::Down,
                dst_direction: HDirection::Down,
            },
        );
        self.draw_direction(None, None);
        self.draw_process(
            "a",
            ComponentConfig {
                ori_direction: HDirection::Down,
                dst_direction: HDirection::Down,
            },
        );
        self.draw_direction(
            None,
            Some(&NCOffset {
                x: offset.x + 100,
                y: offset.y,
            }),
        );
        self.draw_process(
            "a",
            ComponentConfig {
                ori_direction: HDirection::Down,
                dst_direction: HDirection::Up,
            },
        );
    }
}

enum VHDirection {
    Horizontal(HDirection),
    Vertical(VDirection),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum HDirection {
    Up,
    Down,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum VDirection {
    Left,
    Right,
}

trait Direction: Copy + Eq + core::fmt::Debug {}
impl Direction for HDirection {}
impl Direction for VDirection {}

struct ComponentConfig<O: Direction, D: Direction> {
    ori_direction: O,
    dst_direction: D,
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
