use ab_glyph::{Font, FontArc, Glyph, GlyphId, PxScale, ScaleFont, point};
use std::collections::HashMap;

use crate::DirtyRect;

pub struct FontRenderer {
    font: FontArc,
    scale: PxScale,

    glyph_cache: HashMap<GlyphId, CachedGlyph>,
}

struct CachedGlyph {
    bitmap: Vec<u8>, // coverage 0..255
    width: u32,
    height: u32,

    offset_x: i32,
    offset_y: i32,

    advance_x: f32,
}

pub trait FontPixelSink {
    fn blend_pixel(&mut self, x: i32, y: i32, coverage: u8) -> bool;
}

impl FontRenderer {
    pub fn new(font: FontArc, scale: PxScale) -> Self {
        Self {
            font,
            scale,
            glyph_cache: HashMap::new(),
        }
    }

    fn cache_glyph(&mut self, gid: GlyphId) -> Option<&CachedGlyph> {
        if self.glyph_cache.contains_key(&gid) {
            return self.glyph_cache.get(&gid);
        }

        let font = self.font.as_scaled(self.scale);

        let glyph = Glyph {
            id: gid,
            scale: self.scale,
            position: point(0.0, 0.0),
        };

        let outlined = font.outline_glyph(glyph)?;
        let bounds = outlined.px_bounds();

        let bx = bounds.min.x.floor() as i32;
        let by = bounds.min.y.floor() as i32;

        let width = bounds.width().ceil() as u32;
        let height = bounds.height().ceil() as u32;

        let mut bitmap = vec![0u8; (width * height) as usize];

        outlined.draw(|gx, gy, cov| {
            let idx = gy as usize * width as usize + gx as usize;
            bitmap[idx] = (cov * 255.0) as u8;
        });

        let cached = CachedGlyph {
            bitmap,
            width,
            height,
            offset_x: bx,
            offset_y: by,
            advance_x: font.h_advance(gid),
        };

        self.glyph_cache.insert(gid, cached);
        self.glyph_cache.get(&gid)
    }
}

impl FontRenderer {
    pub fn draw_text(&mut self, sink: &mut impl FontPixelSink, x: f32, y: f32, text: &str) -> Option<DirtyRect> {
        if text.is_empty() {
            return None;
        }

        let font = self.font.clone();
        let font = font.as_scaled(self.scale);

        let ascent = font.ascent();
        let baseline_y = (y + ascent).round();

        let mut pen_x = x.round();
        let mut prev: Option<GlyphId> = None;
        let mut dirty: Option<(i32, i32, i32, i32)> = None;

        for ch in text.chars() {
            let gid = font.glyph_id(ch);

            if let Some(p) = prev {
                pen_x += font.kern(p, gid);
            }

            let cached = match self.cache_glyph(gid) {
                Some(g) => g,
                None => {
                    pen_x += font.h_advance(gid);
                    prev = Some(gid);
                    continue;
                }
            };

            let draw_x = pen_x as i32 + cached.offset_x;
            let draw_y = baseline_y as i32 + cached.offset_y;

            Self::blit_glyph(sink, draw_x, draw_y, cached, &mut dirty);

            pen_x += cached.advance_x;
            prev = Some(gid);
        }

        dirty.map(|(min_x, min_y, max_x, max_y)| DirtyRect {
            x: min_x as u32,
            y: min_y as u32,
            w: (max_x - min_x + 1) as u32,
            h: (max_y - min_y + 1) as u32,
        })
    }

    #[inline(always)]
    fn blit_glyph(sink: &mut impl FontPixelSink, gx: i32, gy: i32, glyph: &CachedGlyph, dirty: &mut Option<(i32, i32, i32, i32)>) {
        let w = glyph.width as i32;
        let h = glyph.height as i32;

        for y in 0..h {
            for x in 0..w {
                let cov = glyph.bitmap[(y * w + x) as usize];
                if cov != 0 && sink.blend_pixel(gx + x, gy + y, cov) {
                    let px = gx + x;
                    let py = gy + y;

                    match dirty {
                        Some((min_x, min_y, max_x, max_y)) => {
                            *min_x = (*min_x).min(px);
                            *min_y = (*min_y).min(py);
                            *max_x = (*max_x).max(px);
                            *max_y = (*max_y).max(py);
                        }
                        None => {
                            *dirty = Some((px, py, px, py));
                        }
                    }
                }
            }
        }
    }

    pub fn measure_text_width(&mut self, text: &str) -> f32 {
        let font = self.font.as_scaled(self.scale);

        let mut w = 0.0;
        let mut prev = None;

        for ch in text.chars() {
            let gid = font.glyph_id(ch);

            if let Some(p) = prev {
                w += font.kern(p, gid);
            }

            w += font.h_advance(gid);
            prev = Some(gid);
        }

        w
    }
}
