//! Pixel drawing for charts with too many marks for the DOM. Charts draw into an RGBA
//! buffer here, and adapters copy it onto a `<canvas>` in one call, so a million points
//! cost one boundary crossing rather than a million.
#![allow(clippy::cast_precision_loss, clippy::cast_possible_truncation, clippy::cast_sign_loss, clippy::cast_possible_wrap, reason = "pixel maths on small, clamped sizes")]

use crate::heatmap::HeatLayout;
use crate::scatter::Dot;

/// Above this many marks, charts draw on a canvas instead of one element per mark.
pub const DOM_LIMIT: usize = 2_000;

/// The screen-reader table lists at most this many rows; CSV export still gets them all.
pub const TABLE_LIMIT: usize = 1_000;

/// How many of a table's `rows` (of `cols` cells each) to list: up to [`TABLE_LIMIT`]
/// rows, fewer for wide tables so the table stays under about eight thousand cells.
#[must_use]
pub fn table_rows(rows: usize, cols: usize) -> usize {
    (TABLE_LIMIT * 8 / cols.max(1)).clamp(1, TABLE_LIMIT).min(rows)
}

/// An RGBA colour, not premultiplied.
pub type Rgba = [u8; 4];

/// An RGBA pixel buffer, row by row from the top left, ready for `ImageData`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Raster {
    /// Width in device pixels.
    pub width: usize,
    /// Height in device pixels.
    pub height: usize,
    /// `width * height * 4` bytes.
    pub rgba: Vec<u8>,
}

impl Raster {
    /// A transparent buffer.
    #[must_use]
    pub fn new(width: usize, height: usize) -> Self {
        Self { width, height, rgba: vec![0; width * height * 4] }
    }

    /// Paint `colour` over pixel (`x`, `y`) with extra opacity `cover` (0..=1), source-over.
    fn blend(&mut self, x: usize, y: usize, colour: Rgba, cover: f32) {
        let i = (y * self.width + x) * 4;
        let src_a = f32::from(colour[3]) / 255.0 * cover;
        if src_a <= 0.0 {
            return;
        }
        let px = &mut self.rgba[i..i + 4];
        let dst_a = f32::from(px[3]) / 255.0;
        let out_a = src_a + dst_a * (1.0 - src_a);
        for c in 0..3 {
            let v = (f32::from(colour[c]) * src_a + f32::from(px[c]) * dst_a * (1.0 - src_a)) / out_a;
            px[c] = to_byte(v);
        }
        px[3] = to_byte(out_a * 255.0);
    }

    /// An antialiased disc centred on (`cx`, `cy`) in pixels from the top left.
        pub fn disc(&mut self, cx: f64, cy: f64, r: f64, colour: Rgba) {
        let (w, h) = (self.width as f64, self.height as f64);
        let x0 = (cx - r - 1.0).floor().clamp(0.0, w) as usize;
        let x1 = (cx + r + 1.0).ceil().clamp(0.0, w) as usize;
        let y0 = (cy - r - 1.0).floor().clamp(0.0, h) as usize;
        let y1 = (cy + r + 1.0).ceil().clamp(0.0, h) as usize;
        for y in y0..y1 {
            for x in x0..x1 {
                let d = (x as f64 + 0.5 - cx).hypot(y as f64 + 0.5 - cy);
                let cover = (r + 0.5 - d).clamp(0.0, 1.0);
                if cover > 0.0 {
                    self.blend(x, y, colour, cover as f32);
                }
            }
        }
    }

    /// Fill the pixels from (`x0`, `y0`) up to, not including, (`x1`, `y1`).
    pub fn rect(&mut self, (x0, y0): (usize, usize), (x1, y1): (usize, usize), colour: Rgba) {
        for y in y0..y1.min(self.height) {
            for x in x0..x1.min(self.width) {
                self.blend(x, y, colour, 1.0);
            }
        }
    }
}

fn to_byte(v: f32) -> u8 {
    v.round().clamp(0.0, 255.0) as u8
}

/// Draw scatter `dots` (plot fractions, y up) at `width` × `height` device pixels, with
/// `dpr` device pixels per CSS pixel. `colours[g]` paints group `g` (cycling if short).
/// Dots shrink as the count grows so dense clouds still show their shape.
#[must_use]
pub fn scatter(dots: &[Dot], colours: &[Rgba], width: usize, height: usize, dpr: f64) -> Raster {
    let mut out = Raster::new(width, height);
    if colours.is_empty() {
        return out;
    }
    let base = if dots.len() > 100_000 { 1.25 } else { 2.5 } * dpr;
    let (w, h) = (width as f64, height as f64);
    // Plain dots all share one size, so their coverage is worked out once and stamped.
    let stamp = Stamp::new(base);
    for d in dots {
        let mut c = colours[d.group % colours.len()];
        if d.size.is_some() || dots.len() > 10_000 {
            c[3] = (u16::from(c[3]) * 3 / 4) as u8;
        }
        let (cx, cy) = (d.x * w, (1.0 - d.y) * h);
        match d.size {
            Some(s) => out.disc(cx, cy, (4.0 + s * 16.0) * dpr, c),
            None => stamp.draw(&mut out, cx, cy, c),
        }
    }
    out
}

/// A disc's coverage, precomputed for a pixel-centred dot.
struct Stamp {
    side: usize,
    cover: Vec<f32>,
}

impl Stamp {
    fn new(r: f64) -> Self {
        let side = (2.0 * r + 3.0).ceil() as usize | 1;
        let mid = (side / 2) as f64 + 0.5;
        let cover = (0..side * side)
            .map(|i| {
                let d = ((i % side) as f64 + 0.5 - mid).hypot((i / side) as f64 + 0.5 - mid);
                (r + 0.5 - d).clamp(0.0, 1.0) as f32
            })
            .collect();
        Self { side, cover }
    }

    fn draw(&self, out: &mut Raster, cx: f64, cy: f64, colour: Rgba) {
        let half = (self.side / 2) as isize;
        let (x0, y0) = (cx.floor() as isize - half, cy.floor() as isize - half);
        for (i, &cover) in self.cover.iter().enumerate() {
            let (x, y) = (x0 + (i % self.side) as isize, y0 + (i / self.side) as isize);
            if cover > 0.0 && x >= 0 && y >= 0 && (x as usize) < out.width && (y as usize) < out.height {
                out.blend(x as usize, y as usize, colour, cover);
            }
        }
    }
}

/// Draw a heatmap's cells at `width` × `height` device pixels. `ramp` is the colour scale
/// from lowest to highest level (at least one entry); missing cells stay transparent.
/// Cells get a one-pixel gap once they are at least four pixels across.
#[must_use]
pub fn heat(layout: &HeatLayout, ramp: &[Rgba], width: usize, height: usize) -> Raster {
    let mut out = Raster::new(width, height);
    if ramp.is_empty() || layout.rows == 0 || layout.cols == 0 {
        return out;
    }
    let edge = |i: usize, n: usize, size: usize| i * size / n;
    let gap = usize::from(width / layout.cols >= 4 && height / layout.rows >= 4);
    for cell in &layout.cells {
        if cell.value.is_none() {
            continue;
        }
        let colour = ramp[((cell.level * (ramp.len() - 1) as f64).round() as usize).min(ramp.len() - 1)];
        let (x0, x1) = (edge(cell.col, layout.cols, width), edge(cell.col + 1, layout.cols, width));
        let (y0, y1) = (edge(cell.row, layout.rows, height), edge(cell.row + 1, layout.rows, height));
        out.rect((x0, y0), (x1.saturating_sub(gap).max(x0 + 1), y1.saturating_sub(gap).max(y0 + 1)), colour);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heatmap;

    fn px(r: &Raster, x: usize, y: usize) -> Rgba {
        let i = (y * r.width + x) * 4;
        [r.rgba[i], r.rgba[i + 1], r.rgba[i + 2], r.rgba[i + 3]]
    }

    #[test]
    fn wide_tables_list_fewer_rows() {
        assert_eq!(table_rows(50, 4), 50);
        assert_eq!(table_rows(5_000, 4), TABLE_LIMIT);
        assert_eq!(table_rows(500, 400), 20);
        assert_eq!(table_rows(0, 0), 0);
    }

    #[test]
    fn discs_cover_their_centre_and_not_the_corners() {
        let mut r = Raster::new(10, 10);
        r.disc(5.0, 5.0, 2.0, [255, 0, 0, 255]);
        assert_eq!(px(&r, 5, 5), [255, 0, 0, 255]);
        assert_eq!(px(&r, 0, 0)[3], 0);
    }

    #[test]
    fn blending_mixes_over_what_is_there() {
        let mut r = Raster::new(1, 1);
        r.rect((0, 0), (1, 1), [0, 0, 255, 255]);
        r.rect((0, 0), (1, 1), [255, 0, 0, 128]);
        let p = px(&r, 0, 0);
        assert_eq!(p[3], 255);
        assert!((120..=135).contains(&p[0]) && (120..=135).contains(&p[2]));
    }

    #[test]
    fn scatter_puts_y_up() {
        let dots = [Dot { group: 0, index: 0, x: 0.5, y: 1.0, size: None }];
        let r = scatter(&dots, &[[0, 0, 0, 255]], 20, 20, 1.0);
        assert!(px(&r, 10, 0)[3] > 0);
        assert_eq!(px(&r, 10, 19)[3], 0);
    }

    #[test]
    fn heat_fills_cells_and_skips_missing_ones() {
        let l = heatmap::layout(&[vec![0.0, 1.0], vec![f64::NAN, 1.0]]);
        let r = heat(&l, &[[0, 0, 0, 255], [255, 255, 255, 255]], 20, 20);
        assert_eq!(px(&r, 2, 2), [0, 0, 0, 255]);
        assert_eq!(px(&r, 12, 2), [255, 255, 255, 255]);
        assert_eq!(px(&r, 2, 12)[3], 0);
        assert_eq!(px(&r, 9, 2)[3], 0, "gap between cells");
    }
}
