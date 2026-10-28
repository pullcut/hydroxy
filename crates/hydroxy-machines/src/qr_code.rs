//! QR code encoder (ISO/IEC 18004, model 2). Text goes in as one segment:
//! numeric when it is all digits, alphanumeric when it fits that set, UTF-8
//! bytes otherwise (no ECI, which is what every phone scanner expects). The
//! smallest version from 1 to 40 that fits is chosen, the data gets
//! Reed-Solomon error correction in interleaved blocks, and the mask with the
//! lowest penalty score of all eight is applied.
//!
//! The structure follows Project Nayuki's QR Code generator.

use std::fmt;

/// How much damage the code survives, at the cost of size.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub enum ErrorCorrection {
    /// Recovers about 7% of codewords.
    Low,
    /// Recovers about 15% of codewords.
    #[default]
    Medium,
    /// Recovers about 25% of codewords.
    Quartile,
    /// Recovers about 30% of codewords. Use it under a logo.
    High,
}

impl ErrorCorrection {
    /// All four, weakest first.
    pub const ALL: [Self; 4] = [Self::Low, Self::Medium, Self::Quartile, Self::High];

    /// The letter the standard uses: `L`, `M`, `Q` or `H`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Low => "L",
            Self::Medium => "M",
            Self::Quartile => "Q",
            Self::High => "H",
        }
    }

    const fn index(self) -> usize {
        self as usize
    }

    /// The two format-information bits for this level.
    const fn format_bits(self) -> u32 {
        match self {
            Self::Low => 1,
            Self::Medium => 0,
            Self::Quartile => 3,
            Self::High => 2,
        }
    }
}

/// How the text is packed into bits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mode {
    /// Digits 0–9, about 3.3 bits a character.
    Numeric,
    /// Digits, `A`–`Z`, space and `$%*+-./:`, 5.5 bits a character.
    Alphanumeric,
    /// Raw UTF-8 bytes, 8 bits each.
    Byte,
}

const ALPHANUMERIC: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ $%*+-./:";

impl Mode {
    /// The narrowest mode that can hold all of `text`.
    #[must_use]
    pub fn for_text(text: &str) -> Self {
        if text.bytes().all(|b| b.is_ascii_digit()) {
            Self::Numeric
        } else if text.bytes().all(|b| ALPHANUMERIC.contains(&b)) {
            Self::Alphanumeric
        } else {
            Self::Byte
        }
    }

    const fn indicator(self) -> u32 {
        match self {
            Self::Numeric => 0x1,
            Self::Alphanumeric => 0x2,
            Self::Byte => 0x4,
        }
    }

    /// Width of the character count field at `version`.
    const fn count_bits(self, version: u8) -> usize {
        let band = if version <= 9 {
            0
        } else if version <= 26 {
            1
        } else {
            2
        };
        match self {
            Self::Numeric => [10, 12, 14][band],
            Self::Alphanumeric => [9, 11, 13][band],
            Self::Byte => [8, 16, 16][band],
        }
    }
}

/// Why a code couldn't be made.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QrError {
    /// The text doesn't fit version 40 at this level.
    TooLong,
}

impl fmt::Display for QrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("text is too long for a QR code at this error correction level")
    }
}

impl std::error::Error for QrError {}

// Indexed [level][version]; index 0 is unused.
#[rustfmt::skip]
const ECC_CODEWORDS_PER_BLOCK: [[u8; 41]; 4] = [
    [0, 7, 10, 15, 20, 26, 18, 20, 24, 30, 18, 20, 24, 26, 30, 22, 24, 28, 30, 28, 28, 28, 28, 30, 30, 26, 28, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30],
    [0, 10, 16, 26, 18, 24, 16, 18, 22, 22, 26, 30, 22, 22, 24, 24, 28, 28, 26, 26, 26, 26, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28],
    [0, 13, 22, 18, 26, 18, 24, 18, 22, 20, 24, 28, 26, 24, 20, 30, 24, 28, 28, 26, 30, 28, 30, 30, 30, 30, 28, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30],
    [0, 17, 28, 22, 16, 22, 28, 26, 26, 24, 28, 24, 28, 22, 24, 24, 30, 28, 28, 26, 28, 30, 24, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30],
];

#[rustfmt::skip]
const ERROR_CORRECTION_BLOCKS: [[u8; 41]; 4] = [
    [0, 1, 1, 1, 1, 1, 2, 2, 2, 2, 4, 4, 4, 4, 4, 6, 6, 6, 6, 7, 8, 8, 9, 9, 10, 12, 12, 12, 13, 14, 15, 16, 17, 18, 19, 19, 20, 21, 22, 24, 25],
    [0, 1, 1, 1, 2, 2, 4, 4, 4, 5, 5, 5, 8, 9, 9, 10, 10, 11, 13, 14, 16, 17, 17, 18, 20, 21, 23, 25, 26, 28, 29, 31, 33, 35, 37, 38, 40, 43, 45, 47, 49],
    [0, 1, 1, 2, 2, 4, 4, 6, 6, 8, 8, 8, 10, 12, 16, 12, 17, 16, 18, 21, 20, 23, 23, 25, 27, 29, 34, 34, 35, 38, 40, 43, 45, 48, 51, 53, 56, 59, 62, 65, 68],
    [0, 1, 1, 2, 4, 4, 4, 5, 6, 8, 8, 11, 11, 16, 16, 18, 16, 19, 21, 25, 25, 25, 34, 30, 32, 35, 37, 40, 42, 45, 48, 51, 54, 57, 60, 63, 66, 70, 74, 77, 81],
];

/// Modules left for data and error correction after the function patterns.
fn raw_data_modules(version: u8) -> usize {
    let v = usize::from(version);
    let mut n = (16 * v + 128) * v + 64;
    if v >= 2 {
        let align = v / 7 + 2;
        n -= (25 * align - 10) * align - 55;
        if v >= 7 {
            n -= 36;
        }
    }
    n
}

/// Data codewords (bytes) a version holds at a level, after error correction.
#[must_use]
pub fn data_codewords(version: u8, ecc: ErrorCorrection) -> usize {
    let (v, e) = (usize::from(version), ecc.index());
    raw_data_modules(version) / 8 - usize::from(ECC_CODEWORDS_PER_BLOCK[e][v]) * usize::from(ERROR_CORRECTION_BLOCKS[e][v])
}

/// Bits the text takes in `mode` at `version`, header included.
fn segment_bits(mode: Mode, chars: usize, version: u8) -> usize {
    let data = match mode {
        Mode::Numeric => chars / 3 * 10 + [0, 4, 7][chars % 3],
        Mode::Alphanumeric => chars / 2 * 11 + chars % 2 * 6,
        Mode::Byte => chars * 8,
    };
    4 + mode.count_bits(version) + data
}

/// Bits appended most significant first.
#[derive(Default)]
struct Bits(Vec<bool>);

impl Bits {
    fn push(&mut self, value: u32, len: usize) {
        for i in (0..len).rev() {
            self.0.push((value >> i) & 1 == 1);
        }
    }
}

fn gf_mul(x: u8, y: u8) -> u8 {
    let mut z: u32 = 0;
    for i in (0..8).rev() {
        z = (z << 1) ^ ((z >> 7) * 0x11D);
        z ^= u32::from((y >> i) & 1) * u32::from(x);
    }
    #[allow(clippy::cast_possible_truncation, reason = "reduced modulo the field polynomial, so z < 256")]
    let z = z as u8;
    z
}

/// The generator polynomial of `degree`, highest coefficient dropped.
fn rs_divisor(degree: usize) -> Vec<u8> {
    let mut result = vec![0u8; degree];
    result[degree - 1] = 1;
    let mut root = 1u8;
    for _ in 0..degree {
        for j in 0..degree {
            result[j] = gf_mul(result[j], root);
            if j + 1 < degree {
                result[j] ^= result[j + 1];
            }
        }
        root = gf_mul(root, 0x02);
    }
    result
}

fn rs_remainder(data: &[u8], divisor: &[u8]) -> Vec<u8> {
    let mut result = vec![0u8; divisor.len()];
    for &b in data {
        let factor = b ^ result.remove(0);
        result.push(0);
        for (r, &d) in result.iter_mut().zip(divisor) {
            *r ^= gf_mul(d, factor);
        }
    }
    result
}

/// A finished QR code: a square of dark (`true`) and light modules, without the quiet zone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QrCode {
    version: u8,
    size: usize,
    ecc: ErrorCorrection,
    mode: Mode,
    mask: u8,
    modules: Vec<bool>,
    function: Vec<bool>,
}

impl QrCode {
    /// Encode `text` at `ecc` in the smallest version that fits, with the best mask.
    ///
    /// # Errors
    /// [`QrError::TooLong`] when it doesn't fit version 40.
    pub fn encode(text: &str, ecc: ErrorCorrection) -> Result<Self, QrError> {
        Self::encode_with(text, ecc, 1, None)
    }

    /// [`encode`](Self::encode) with a smallest version (1–40) and an optional fixed mask (0–7).
    ///
    /// # Errors
    /// [`QrError::TooLong`] when it doesn't fit version 40.
    pub fn encode_with(text: &str, ecc: ErrorCorrection, min_version: u8, mask: Option<u8>) -> Result<Self, QrError> {
        let mode = Mode::for_text(text);
        let chars = if mode == Mode::Byte { text.len() } else { text.chars().count() };
        let version = (min_version.clamp(1, 40)..=40)
            .find(|&v| chars < (1 << mode.count_bits(v)) && segment_bits(mode, chars, v) <= data_codewords(v, ecc) * 8)
            .ok_or(QrError::TooLong)?;

        let mut bits = Bits::default();
        bits.push(mode.indicator(), 4);
        bits.push(u32::try_from(chars).map_err(|_| QrError::TooLong)?, mode.count_bits(version));
        let bytes = text.as_bytes();
        match mode {
            Mode::Numeric => {
                for group in bytes.chunks(3) {
                    let n = group.iter().fold(0, |n, &d| n * 10 + u32::from(d - b'0'));
                    bits.push(n, group.len() * 3 + 1);
                }
            }
            Mode::Alphanumeric => {
                let value = |b: &u8| ALPHANUMERIC.iter().position(|c| c == b).and_then(|p| u32::try_from(p).ok()).unwrap_or(0);
                for pair in bytes.chunks(2) {
                    match pair {
                        [a, b] => bits.push(value(a) * 45 + value(b), 11),
                        [a] => bits.push(value(a), 6),
                        _ => {}
                    }
                }
            }
            Mode::Byte => bytes.iter().for_each(|&b| bits.push(u32::from(b), 8)),
        }
        let capacity = data_codewords(version, ecc) * 8;
        bits.push(0, (capacity - bits.0.len()).min(4));
        bits.push(0, (8 - bits.0.len() % 8) % 8);
        let mut data: Vec<u8> = bits.0.chunks(8).map(|byte| byte.iter().fold(0u8, |acc, &b| acc << 1 | u8::from(b))).collect();
        for pad in [0xEC, 0x11].into_iter().cycle() {
            if data.len() * 8 >= capacity {
                break;
            }
            data.push(pad);
        }

        let size = usize::from(version) * 4 + 17;
        let mut qr = Self { version, size, ecc, mode, mask: 0, modules: vec![false; size * size], function: vec![false; size * size] };
        qr.draw_function_patterns();
        let codewords = qr.add_error_correction(&data);
        qr.draw_codewords(&codewords);

        let mask = mask.map_or_else(
            || {
                (0..8)
                    .min_by_key(|&m| {
                        qr.apply_mask(m);
                        qr.draw_format_bits(m);
                        let p = qr.penalty();
                        qr.apply_mask(m);
                        p
                    })
                    .unwrap_or(0)
            },
            |m| m.min(7),
        );
        qr.mask = mask;
        qr.apply_mask(mask);
        qr.draw_format_bits(mask);
        Ok(qr)
    }

    /// Version, 1–40.
    #[must_use]
    pub const fn version(&self) -> u8 {
        self.version
    }

    /// Modules per side: `version * 4 + 17`.
    #[must_use]
    pub const fn size(&self) -> usize {
        self.size
    }

    /// The error correction level used.
    #[must_use]
    pub const fn ecc(&self) -> ErrorCorrection {
        self.ecc
    }

    /// How the text was packed.
    #[must_use]
    pub const fn mode(&self) -> Mode {
        self.mode
    }

    /// The mask applied, 0–7.
    #[must_use]
    pub const fn mask(&self) -> u8 {
        self.mask
    }

    /// Whether the module at column `x`, row `y` is dark. Outside the square is light.
    #[must_use]
    pub fn module(&self, x: usize, y: usize) -> bool {
        x < self.size && y < self.size && self.modules[y * self.size + x]
    }

    fn set_function(&mut self, x: usize, y: usize, dark: bool) {
        self.modules[y * self.size + x] = dark;
        self.function[y * self.size + x] = true;
    }

    fn alignment_positions(&self) -> Vec<usize> {
        if self.version == 1 {
            return Vec::new();
        }
        let v = usize::from(self.version);
        let count = v / 7 + 2;
        let step = (v * 8 + count * 3 + 5) / (count * 4 - 4) * 2;
        let mut result: Vec<usize> = (0..count - 1).map(|i| self.size - 7 - i * step).collect();
        result.push(6);
        result.reverse();
        result
    }

    fn draw_function_patterns(&mut self) {
        let size = self.size;
        for i in 0..size {
            self.set_function(6, i, i % 2 == 0);
            self.set_function(i, 6, i % 2 == 0);
        }
        for (cx, cy) in [(3, 3), (size - 4, 3), (3, size - 4)] {
            for dy in -4_isize..=4 {
                for dx in -4_isize..=4 {
                    let (x, y) = (cx.checked_add_signed(dx), cy.checked_add_signed(dy));
                    if let (Some(x), Some(y)) = (x, y)
                        && x < size
                        && y < size
                    {
                        let dist = dx.abs().max(dy.abs());
                        self.set_function(x, y, dist != 2 && dist != 4);
                    }
                }
            }
        }
        let align = self.alignment_positions();
        let last = align.len().saturating_sub(1);
        for (i, &ay) in align.iter().enumerate() {
            for (j, &ax) in align.iter().enumerate() {
                // Skip the three corners the finders occupy.
                if (i == 0 && (j == 0 || j == last)) || (i == last && j == 0) {
                    continue;
                }
                for dy in -2_isize..=2 {
                    for dx in -2_isize..=2 {
                        self.set_function(ax.wrapping_add_signed(dx), ay.wrapping_add_signed(dy), dx.abs().max(dy.abs()) != 1);
                    }
                }
            }
        }
        self.draw_format_bits(0);
        if self.version >= 7 {
            let bits = version_bits(self.version);
            for i in 0..18 {
                let dark = (bits >> i) & 1 == 1;
                let (a, b) = (size - 11 + i % 3, i / 3);
                self.set_function(a, b, dark);
                self.set_function(b, a, dark);
            }
        }
    }

    fn draw_format_bits(&mut self, mask: u8) {
        let bits = format_bits(self.ecc, mask);
        let bit = |i: usize| (bits >> i) & 1 == 1;
        let size = self.size;
        for i in 0..=5 {
            self.set_function(8, i, bit(i));
        }
        self.set_function(8, 7, bit(6));
        self.set_function(8, 8, bit(7));
        self.set_function(7, 8, bit(8));
        for i in 9..15 {
            self.set_function(14 - i, 8, bit(i));
        }
        for i in 0..8 {
            self.set_function(size - 1 - i, 8, bit(i));
        }
        for i in 8..15 {
            self.set_function(8, size - 15 + i, bit(i));
        }
        self.set_function(8, size - 8, true);
    }

    fn add_error_correction(&self, data: &[u8]) -> Vec<u8> {
        let (v, e) = (usize::from(self.version), self.ecc.index());
        let blocks = usize::from(ERROR_CORRECTION_BLOCKS[e][v]);
        let ecc_len = usize::from(ECC_CODEWORDS_PER_BLOCK[e][v]);
        let raw = raw_data_modules(self.version) / 8;
        let short_blocks = blocks - raw % blocks;
        let short_len = raw / blocks;
        let divisor = rs_divisor(ecc_len);
        let mut out_blocks = Vec::with_capacity(blocks);
        let mut k = 0;
        for i in 0..blocks {
            let len = short_len - ecc_len + usize::from(i >= short_blocks);
            let mut block = data[k..k + len].to_vec();
            k += len;
            let ecc = rs_remainder(&block, &divisor);
            if i < short_blocks {
                block.push(0);
            }
            block.extend(ecc);
            out_blocks.push(block);
        }
        let mut result = Vec::with_capacity(raw);
        for i in 0..=short_len {
            for (j, block) in out_blocks.iter().enumerate() {
                if i != short_len - ecc_len || j >= short_blocks {
                    result.push(block[i]);
                }
            }
        }
        result
    }

    fn draw_codewords(&mut self, data: &[u8]) {
        let size = self.size;
        let mut i = 0;
        let mut right = size - 1;
        while right >= 1 {
            if right == 6 {
                right = 5;
            }
            for vert in 0..size {
                for j in 0..2 {
                    let x = right - j;
                    let upward = (right + 1) & 2 == 0;
                    let y = if upward { size - 1 - vert } else { vert };
                    if !self.function[y * size + x] && i < data.len() * 8 {
                        self.modules[y * size + x] = (data[i >> 3] >> (7 - (i & 7))) & 1 == 1;
                        i += 1;
                    }
                }
            }
            if right < 2 {
                break;
            }
            right -= 2;
        }
    }

    /// XOR mask `mask` onto the data modules. Applying it twice undoes it.
    fn apply_mask(&mut self, mask: u8) {
        let size = self.size;
        for y in 0..size {
            for x in 0..size {
                let flip = match mask {
                    0 => (x + y) % 2 == 0,
                    1 => y % 2 == 0,
                    2 => x % 3 == 0,
                    3 => (x + y) % 3 == 0,
                    4 => (x / 3 + y / 2) % 2 == 0,
                    5 => x * y % 2 + x * y % 3 == 0,
                    6 => (x * y % 2 + x * y % 3) % 2 == 0,
                    _ => ((x + y) % 2 + x * y % 3) % 2 == 0,
                };
                if flip && !self.function[y * size + x] {
                    self.modules[y * size + x] ^= true;
                }
            }
        }
    }

    /// The ISO 18004 penalty score: runs, 2×2 blocks, finder look-alikes and dark balance.
    fn penalty(&self) -> usize {
        let size = self.size;
        let mut result = 0;
        for line in 0..2 {
            for a in 0..size {
                let at = |b: usize| if line == 0 { self.modules[a * size + b] } else { self.modules[b * size + a] };
                let mut run_color = false;
                let mut run = 0;
                let mut history = [0usize; 7];
                for b in 0..size {
                    if at(b) == run_color {
                        run += 1;
                        if run == 5 {
                            result += 3;
                        } else if run > 5 {
                            result += 1;
                        }
                    } else {
                        add_history(run, &mut history, size);
                        if !run_color {
                            result += finder_count(&history) * 40;
                        }
                        run_color = at(b);
                        run = 1;
                    }
                }
                if run_color {
                    add_history(run, &mut history, size);
                    run = 0;
                }
                add_history(run + size, &mut history, size);
                result += finder_count(&history) * 40;
            }
        }
        for y in 0..size - 1 {
            for x in 0..size - 1 {
                let c = self.modules[y * size + x];
                if c == self.modules[y * size + x + 1] && c == self.modules[(y + 1) * size + x] && c == self.modules[(y + 1) * size + x + 1] {
                    result += 3;
                }
            }
        }
        let dark = self.modules.iter().filter(|&&m| m).count();
        let total = size * size;
        let k = (dark * 20).abs_diff(total * 10).div_ceil(total).saturating_sub(1);
        result + k * 10
    }
}

fn add_history(mut run: usize, history: &mut [usize; 7], size: usize) {
    if history[0] == 0 {
        run += size; // the light border before the first run
    }
    history.copy_within(0..6, 1);
    history[0] = run;
}

/// 1:1:3:1:1 dark-light runs with four light modules on either side.
fn finder_count(h: &[usize; 7]) -> usize {
    let n = h[1];
    let core = n > 0 && h[2] == n && h[3] == n * 3 && h[4] == n && h[5] == n;
    usize::from(core && h[0] >= n * 4 && h[6] >= n) + usize::from(core && h[6] >= n * 4 && h[0] >= n)
}

/// The 15 format bits for a level and mask: BCH(15,5), XOR 0x5412.
#[must_use]
pub fn format_bits(ecc: ErrorCorrection, mask: u8) -> u32 {
    let data = ecc.format_bits() << 3 | u32::from(mask & 7);
    let mut rem = data;
    for _ in 0..10 {
        rem = (rem << 1) ^ ((rem >> 9) * 0x537);
    }
    (data << 10 | rem) ^ 0x5412
}

/// The 18 version bits for versions 7 and up: BCH(18,6).
#[must_use]
pub fn version_bits(version: u8) -> u32 {
    let mut rem = u32::from(version);
    for _ in 0..12 {
        rem = (rem << 1) ^ ((rem >> 11) * 0x1F25);
    }
    u32::from(version) << 12 | rem
}

#[cfg(test)]
mod tests;
