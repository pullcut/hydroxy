//! Checks against matrices from segno, a reference encoder, plus structural checks.

use super::*;

/// (text, level, version, mask, rows): reference matrices from segno 1.6.6, no quiet zone.
///
/// segno 1.6.6 adds a whole zero byte of "padding" when the terminator ends on a byte
/// boundary, where ISO 18004 7.4.10 adds none, so these come from segno with
/// `write_padding_bits` patched to pad `(8 - len % 8) % 8` bits.
const REFERENCE: &[(&str, ErrorCorrection, u8, u8, &str)] = &[
    (
        "01234567",
        ErrorCorrection::Medium,
        1,
        3,
        "\
        #######.##.##.#######\
        #.....#.#.#...#.....#\
        #.###.#..##.#.#.###.#\
        #.###.#.##....#.###.#\
        #.###.#..##...#.###.#\
        #.....#..##...#.....#\
        #######.#.#.#.#######\
        ........##...........\
        #.##.###..#...#..#.##\
        ...#.#.##.#.#..#.##..\
        #..#.####...#####..#.\
        ##.#...#..#.##...#.#.\
        ...######..#.#..#....\
        ........###..#.#....#\
        #######.#....##.#.##.\
        #.....#.#.#####...#.#\
        #.###.#..#.#..#.....#\
        #.###.#.#.#..#..#.##.\
        #.###.#.#.##.#..#.#..\
        #.....#..#.##.#.##.##\
        #######.#..##..#...#.\
    ",
    ),
    (
        "HELLO WORLD",
        ErrorCorrection::Quartile,
        1,
        1,
        "\
        #######....#..#######\
        #.....#..#....#.....#\
        #.###.#..#..#.#.###.#\
        #.###.#.##.#..#.###.#\
        #.###.#..###..#.###.#\
        #.....#.####..#.....#\
        #######.#.#.#.#######\
        ........##.#.........\
        .##...#..#.##.##.#...\
        ...#.#.##.#..#.###.##\
        .##...#...##.####..#.\
        ..###....##.......#..\
        ##.########.###.#####\
        ........#....###.####\
        #######..###.#....##.\
        #.....#.....###....#.\
        #.###.#..###.##.#.#.#\
        #.###.#..........#...\
        #.###.#.##....#....##\
        #.....#.###.#..#....#\
        #######..#....#..#.##\
    ",
    ),
    (
        "https://oxide.rs",
        ErrorCorrection::Low,
        1,
        3,
        "\
        #######.##.#..#######\
        #.....#..##...#.....#\
        #.###.#.#.#.#.#.###.#\
        #.###.#.##..#.#.###.#\
        #.###.#.###.#.#.###.#\
        #.....#....#..#.....#\
        #######.#.#.#.#######\
        .........####........\
        ####..#.#.####..###.#\
        ##.#.#.####..########\
        ...#..##..##.#...#.##\
        #.#.##.###.#...#.#.#.\
        .#.#..##...##.#.##..#\
        ........######..#....\
        #######....#..#.#....\
        #.....#..#..##.####..\
        #.###.#......#..#.#.#\
        #.###.#.#...####.....\
        #.###.#.#####..#..#..\
        #.....#.##...####...#\
        #######.#.#.###.###..\
    ",
    ),
    (
        "€5 café",
        ErrorCorrection::High,
        2,
        6,
        "\
        #######..####.....#######\
        #.....#...#######.#.....#\
        #.###.#.#.##.#..#.#.###.#\
        #.###.#.###.####..#.###.#\
        #.###.#..#.#..#...#.###.#\
        #.....#...##...##.#.....#\
        #######.#.#.#.#.#.#######\
        ..........#######........\
        ...##.##...#...#.....##..\
        .#####..###.#.#.#....#...\
        ####..##.#####.#.#.#.#.#.\
        #.###...##.....#..#...##.\
        .###.##...###....####..##\
        ###.#..#..#.###...#..#..#\
        ##...####.#.#..#.#.##.#..\
        #...##.##.#..#..#..##.#..\
        #...#.##.#....###########\
        ........#.####..#...#.##.\
        #######.#.###..##.#.###..\
        #.....#....#.####...##...\
        #.###.#.####.##.######...\
        #.###.#.#.#.#.#..#....#.#\
        #.###.#...#.##..#####.###\
        #.....#..###..######.####\
        #######..###...##.####..#\
    ",
    ),
    (
        "Meet at the north gate of the Barbican at half past seven; bring the tickets and an umbrella.",
        ErrorCorrection::High,
        9,
        2,
        "\
        #######.#####...#..#.#..#.........####....#...#######\
        #.....#.#..#....###..#..#..#.##..###.###..##..#.....#\
        #.###.#.##.####.#.#####.####..#.##..#...#..#..#.###.#\
        #.###.#...#...#..#..#..#.#.##.#...##.#....#.#.#.###.#\
        #.###.#..###....#.##...#######..###..#...##...#.###.#\
        #.....#.#.#####.#.###...#...######.#..##..#...#.....#\
        #######.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#######\
        ........##.##.###.#.#...#...#..####..#..###.#........\
        ..###.#.###.........###.#####.....####...##.####..###\
        ...#.#.#.#.##.######.#...###..#..#####.#..##.#.#...##\
        ##.####..##.##....##.###.#..#.##..###.##.....##.###..\
        #####..#..##.#.##...#.......#.#.#......#.##......#..#\
        ..#####.#.#..##.#.#...###.####..#..##...#.#.#..##.#.#\
        ##.#.#.##.##.#.####.#####.##..#...##.#.##.##.......##\
        ##..###.##..#..##...#.##.#.##...#...###.#....###.....\
        .###.#..#.#..#.###...#.#.#.###.#.##....##.###..#.#.#.\
        ##....#####...##.#####.#..#..##...####...#..#..##.##.\
        ..#.#......#.#.##.#.##.##...#####.#.##.#.###...#...#.\
        ####..#####.#####..###.##...#...#...#.##...#..#.#.#..\
        ..#..#....###.###.######..#.#..##..##..####..#...#...\
        ...#.##..#.####.######..####...###..##.#..#####.#.#.#\
        .......##.#.#...#.###..#.##..#.#.####....###.....#.##\
        #####.#..###.#.#...##...###.#.###...##........##.#...\
        .#..##.#.###.#.###...###.#..##.....#.#.###.....#.#...\
        ##.######....#..#....########.##.####......########.#\
        .####...###.#..###.#..###...#.#####.##.######...#..##\
        #.#.#.#.#.######...##.###.#.###...#.#.#....##.#.#.#..\
        ..#.#...##..#..####..#..#...#...#....#.##.#.#...##..#\
        .##.#####..##...##...########...#..##....##.#########\
        .#.#...###.##..#.#..##..##..##.#.#.###.#..#..#.####.#\
        ##..#.#.....##..#.....##.#.#..####....###...#...##...\
        .#.....##.#..#.#.##.#.####.#.#.##......##.#####.##.##\
        .###..###.######.#..#..#..#..####...#.#..##.##.#..##.\
        #.##.#......#.#......###.##.###.####.#.####..#.####.#\
        #...#.#...#.##.#.##..#..###..##..##...##.#.....#..#..\
        .#.###...#.###.#.#.....####.....###..####.######.#.#.\
        ......##...####.#..#####.#...#.##.#.#.....#.......##.\
        .#..#...#.....##....###..#..####.#...#..###..#.###.##\
        #.##.###...###..###.#.#...#...#..####.#....#.#.##.#..\
        ###.##..#.#...##.#..###.####..###....####.#######....\
        .#######.#....####....#..#....#.#.###.##..#.......###\
        ...##..##.##.#.#..#..##...#..##.#......#.##.#.###..##\
        ##.#####.###..#......#....##...#.#..###...#..#...#...\
        .##.....##.##.#..#...####...##.#..#..#.#..##.##.##.#.\
        ...#..#....#...##.#.#.#.#####.#.##..#..#..#.#####.##.\
        ........##..####.##.#...#...#.##.#.###.####.#...#..##\
        #######...##..#.###.#..##.#.#..######.#.#...#.#.##.#.\
        #.....#..#####..####....#...##...##..########...##...\
        #.###.#.###.#..##.###.###########..##.##.##.#######..\
        #.###.#.#.##..#....###.##..#.###.#..#..#.##.##.#..#..\
        #.###.#.####.#.#####..#.##...#.##.....#..#.#.#.###...\
        #.....#....###.#........##..###...#.#####.......##.#.\
        #######..#.##...####.#.###...#####.#..#...#######.#..\
    ",
    ),
];

fn rows(qr: &QrCode) -> String {
    (0..qr.size()).flat_map(|y| (0..qr.size()).map(move |x| if qr.module(x, y) { '#' } else { '.' })).collect()
}

fn fnv(qr: &QrCode) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for y in 0..qr.size() {
        for x in 0..qr.size() {
            h ^= u64::from(qr.module(x, y));
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
    }
    h
}

#[test]
fn matches_the_reference_module_for_module() {
    for &(text, ecc, version, mask, expected) in REFERENCE {
        let fixed = QrCode::encode_with(text, ecc, 1, Some(mask)).unwrap();
        assert_eq!(fixed.version(), version, "{text}");
        assert_eq!(rows(&fixed), expected, "{text} with mask {mask}");
        // Masks are chosen by penalty. segno scores before adding format bits and only
        // counts 1:1:3:1:1 at unit width; ISO's table says "ratio", and ZXing, libqrencode
        // and Nayuki score the finished symbol at any width, as this encoder does. Both
        // are valid symbols, so only check the choice is our own minimum and repeatable.
        let auto = QrCode::encode(text, ecc).unwrap();
        let best = (0..8).map(|m| QrCode::encode_with(text, ecc, 1, Some(m)).unwrap().penalty()).min().unwrap();
        assert_eq!(auto.penalty(), best, "{text}");
        assert_eq!(auto, QrCode::encode_with(text, ecc, 1, Some(auto.mask())).unwrap());
    }
}

#[test]
fn bigger_versions_match_the_reference() {
    let cases: [(String, ErrorCorrection, u8, u8, u8, u64); 5] = [
        ("1234567890".repeat(60), ErrorCorrection::Medium, 1, 11, 4, 0x6074_d255_ac42_d41d),
        (
            "THE QUICK BROWN FOX JUMPS OVER THE LAZY DOG 0123456789 $%*+-./:".repeat(12),
            ErrorCorrection::Quartile,
            1,
            22,
            3,
            0xd36a_c550_1d44_005f,
        ),
        (
            (b'a'..=b'z').cycle().take(1200).map(char::from).collect(),
            ErrorCorrection::Low,
            1,
            25,
            2,
            0x89bb_96ee_f3ae_1c7f,
        ),
        ("x".into(), ErrorCorrection::High, 40, 40, 4, 0xea2c_5f0e_196d_15f3),
        ("Grüße aus Köln, 東京 und Zürich! ".repeat(20), ErrorCorrection::High, 1, 31, 0, 0x17f4_b0ae_1aeb_acc5),
    ];
    for (text, ecc, min, version, mask, hash) in cases {
        let qr = QrCode::encode_with(&text, ecc, min, Some(mask)).unwrap();
        assert_eq!((qr.version(), qr.mask()), (version, mask), "{text}");
        assert_eq!(fnv(&qr), hash, "{text}");
    }
}

#[test]
fn picks_the_narrowest_mode() {
    assert_eq!(Mode::for_text("0123"), Mode::Numeric);
    assert_eq!(Mode::for_text("HELLO WORLD"), Mode::Alphanumeric);
    assert_eq!(Mode::for_text("Hello"), Mode::Byte);
    assert_eq!(Mode::for_text("ÉTÉ"), Mode::Byte);
}

#[test]
fn function_patterns_are_in_place() {
    let qr = QrCode::encode("https://oxide.rs/docs/qr-code", ErrorCorrection::Quartile).unwrap();
    let n = qr.size();
    for (ox, oy) in [(0, 0), (n - 7, 0), (0, n - 7)] {
        for dy in 0..7_usize {
            for dx in 0..7_usize {
                let dist = dx.abs_diff(3).max(dy.abs_diff(3));
                assert_eq!(qr.module(ox + dx, oy + dy), dist != 2, "finder at ({ox}, {oy})");
            }
        }
    }
    // Separators: light around each finder.
    for i in 0..8 {
        assert!(!qr.module(7, i) && !qr.module(i, 7));
        assert!(!qr.module(n - 8, i) && !qr.module(n - 1 - i, 7));
        assert!(!qr.module(7, n - 1 - i) && !qr.module(i, n - 8));
    }
    // Timing patterns alternate between the finders.
    for i in 8..n - 8 {
        assert_eq!(qr.module(i, 6), i % 2 == 0);
        assert_eq!(qr.module(6, i), i % 2 == 0);
    }
    assert!(qr.module(8, n - 8), "dark module");
}

/// Read the first format copy and correct it to the nearest valid word, as a scanner would.
fn read_format(qr: &QrCode) -> (ErrorCorrection, u8) {
    let mut bits = 0u32;
    let mut set = |i: usize, dark: bool| bits |= u32::from(dark) << i;
    for i in 0..=5 {
        set(i, qr.module(8, i));
    }
    set(6, qr.module(8, 7));
    set(7, qr.module(8, 8));
    set(8, qr.module(7, 8));
    for i in 9..15 {
        set(i, qr.module(14 - i, 8));
    }
    // The second copy must agree.
    let n = qr.size();
    let mut second = 0u32;
    for i in 0..8 {
        second |= u32::from(qr.module(n - 1 - i, 8)) << i;
    }
    for i in 8..15 {
        second |= u32::from(qr.module(8, n - 15 + i)) << i;
    }
    assert_eq!(bits, second);
    let (ecc, mask) = ErrorCorrection::ALL
        .into_iter()
        .flat_map(|e| (0..8).map(move |m| (e, m)))
        .min_by_key(|&(e, m)| (format_bits(e, m) ^ bits).count_ones())
        .unwrap();
    assert_eq!(format_bits(ecc, mask), bits, "no bit errors in a fresh code");
    (ecc, mask)
}

#[test]
fn format_bits_decode_to_the_level_and_mask() {
    // Known words from the standard's table.
    assert_eq!(format_bits(ErrorCorrection::Medium, 0), 0b101_0100_0001_0010);
    assert_eq!(format_bits(ErrorCorrection::Low, 4), 0b110_0110_0010_1111);
    for ecc in ErrorCorrection::ALL {
        for mask in 0..8 {
            let qr = QrCode::encode_with("oxide", ecc, 1, Some(mask)).unwrap();
            assert_eq!(read_format(&qr), (ecc, mask));
        }
    }
    // Every pair of format words differs in at least 7 bits, so 3 errors correct.
    let words: Vec<u32> =
        ErrorCorrection::ALL.into_iter().flat_map(|e| (0..8).map(move |m| format_bits(e, m))).collect();
    for (i, a) in words.iter().enumerate() {
        for b in &words[i + 1..] {
            assert!((a ^ b).count_ones() >= 7);
        }
    }
}

#[test]
fn version_info_decodes_for_seven_and_up() {
    assert_eq!(version_bits(7), 0x07C94);
    assert_eq!(version_bits(40), 0x28C69);
    let qr = QrCode::encode_with("v7", ErrorCorrection::Low, 7, None).unwrap();
    assert_eq!(qr.version(), 7);
    let n = qr.size();
    let (mut a, mut b) = (0u32, 0u32);
    for i in 0..18 {
        a |= u32::from(qr.module(n - 11 + i % 3, i / 3)) << i;
        b |= u32::from(qr.module(i / 3, n - 11 + i % 3)) << i;
    }
    assert_eq!((a, b), (version_bits(7), version_bits(7)));
    assert_eq!(a >> 12, 7);
}

#[test]
fn capacity_limits() {
    // Version 40 maxima from the standard's capacity table.
    let max = [
        (ErrorCorrection::Low, Mode::Numeric, 7089),
        (ErrorCorrection::Low, Mode::Alphanumeric, 4296),
        (ErrorCorrection::Low, Mode::Byte, 2953),
        (ErrorCorrection::High, Mode::Numeric, 3057),
        (ErrorCorrection::High, Mode::Byte, 1273),
    ];
    for (ecc, mode, n) in max {
        let ch = match mode {
            Mode::Numeric => "7",
            Mode::Alphanumeric => "A",
            Mode::Byte => "a",
        };
        let qr = QrCode::encode(&ch.repeat(n), ecc).unwrap();
        assert_eq!((qr.version(), qr.mode()), (40, mode));
        assert_eq!(QrCode::encode(&ch.repeat(n + 1), ecc), Err(QrError::TooLong));
    }
    // Version 1 limits: 41 digits at L, 17 bytes at L, 7 bytes at H.
    assert_eq!(QrCode::encode(&"1".repeat(41), ErrorCorrection::Low).unwrap().version(), 1);
    assert_eq!(QrCode::encode(&"1".repeat(42), ErrorCorrection::Low).unwrap().version(), 2);
    assert_eq!(QrCode::encode(&"a".repeat(17), ErrorCorrection::Low).unwrap().version(), 1);
    assert_eq!(QrCode::encode(&"a".repeat(7), ErrorCorrection::High).unwrap().version(), 1);
    assert_eq!(QrCode::encode(&"a".repeat(8), ErrorCorrection::High).unwrap().version(), 2);
    // Data codewords per version agree with the standard's table at a few spots.
    assert_eq!(data_codewords(1, ErrorCorrection::Medium), 16);
    assert_eq!(data_codewords(10, ErrorCorrection::Quartile), 154);
    assert_eq!(data_codewords(40, ErrorCorrection::Low), 2956);
}

#[test]
fn reed_solomon_matches_the_textbook_example() {
    // "HELLO WORLD" 1-M data codewords and their ECC from Thonky's tutorial.
    let data = [32, 91, 11, 120, 209, 114, 220, 77, 67, 64, 236, 17, 236, 17, 236, 17];
    assert_eq!(rs_remainder(&data, &rs_divisor(10)), [196, 35, 39, 119, 235, 215, 231, 226, 93, 23]);
}
