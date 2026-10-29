//! Country table, grouping and E.164 parsing.

use crate::listbox;

/// A country's dialling plan, simplified to what grouping and validity need.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Country {
    /// ISO 3166-1 alpha-2 code, such as `GB`.
    pub iso: &'static str,
    /// English name.
    pub name: &'static str,
    /// Dial code without the `+`, such as `44`.
    pub dial: &'static str,
    /// Grouping for the national significant number: `#` is a digit, anything else a literal.
    pub pattern: &'static str,
    /// An example national significant number, for the placeholder.
    pub example: &'static str,
    /// Fewest digits in a valid national significant number.
    pub min: u8,
    /// Most digits in a valid national significant number.
    pub max: u8,
    /// A leading `0` is a domestic trunk prefix, typed at home and dropped in E.164.
    pub trunk: bool,
}

#[allow(clippy::too_many_arguments, reason = "one row of the table")]
const fn c(
    iso: &'static str,
    name: &'static str,
    dial: &'static str,
    pattern: &'static str,
    example: &'static str,
    min: u8,
    max: u8,
    trunk: bool,
) -> Country {
    Country { iso, name, dial, pattern, example, min, max, trunk }
}

// length ranges and one grouping per country, not libphonenumber's full
// per-prefix metadata. Mobile-first examples; fixed lines of another length still
// validate but may group a little differently from local habit.
/// The countries the picker offers, sorted by name.
pub const COUNTRIES: &[Country] = &[
    c("AR", "Argentina", "54", "## ####-####", "1123456789", 10, 10, true),
    c("AU", "Australia", "61", "### ### ###", "412345678", 9, 9, true),
    c("AT", "Austria", "43", "### #######", "6641234567", 4, 13, true),
    c("BH", "Bahrain", "973", "#### ####", "36001234", 8, 8, false),
    c("BD", "Bangladesh", "880", "####-######", "1812345678", 10, 10, true),
    c("BE", "Belgium", "32", "### ## ## ##", "470123456", 8, 9, true),
    c("BR", "Brazil", "55", "(##) #####-####", "11961234567", 10, 11, false),
    c("BG", "Bulgaria", "359", "## ### ####", "48123456", 8, 9, true),
    c("CA", "Canada", "1", "(###) ###-####", "6135550123", 10, 10, false),
    c("CL", "Chile", "56", "# #### ####", "221234567", 9, 9, false),
    c("CN", "China", "86", "### #### ####", "13123456789", 11, 11, false),
    c("CO", "Colombia", "57", "### #######", "3211234567", 10, 10, false),
    c("CR", "Costa Rica", "506", "#### ####", "83123456", 8, 8, false),
    c("HR", "Croatia", "385", "## ### ####", "921234567", 8, 9, true),
    c("CZ", "Czechia", "420", "### ### ###", "601123456", 9, 9, false),
    c("DK", "Denmark", "45", "## ## ## ##", "32123456", 8, 8, false),
    c("EG", "Egypt", "20", "### ### ####", "1001234567", 10, 10, true),
    c("FI", "Finland", "358", "## ### ####", "412345678", 5, 12, true),
    c("FR", "France", "33", "# ## ## ## ##", "612345678", 9, 9, true),
    c("DE", "Germany", "49", "### ########", "15123456789", 6, 11, true),
    c("GH", "Ghana", "233", "## ### ####", "231234567", 9, 9, true),
    c("GR", "Greece", "30", "### ### ####", "6912345678", 10, 10, false),
    c("HK", "Hong Kong", "852", "#### ####", "51234567", 8, 8, false),
    c("HU", "Hungary", "36", "## ### ####", "201234567", 8, 9, false),
    c("IS", "Iceland", "354", "### ####", "6111234", 7, 7, false),
    c("IN", "India", "91", "#####-#####", "8123456789", 10, 10, true),
    c("ID", "Indonesia", "62", "###-####-####", "812345678", 9, 12, true),
    c("IE", "Ireland", "353", "## ### ####", "851234567", 7, 9, true),
    c("IL", "Israel", "972", "##-###-####", "502345678", 8, 9, true),
    c("IT", "Italy", "39", "### ### ####", "3123456789", 6, 11, false),
    c("JP", "Japan", "81", "##-####-####", "9012345678", 9, 10, true),
    c("KE", "Kenya", "254", "### ######", "712123456", 9, 9, true),
    c("KW", "Kuwait", "965", "#### ####", "50012345", 8, 8, false),
    c("LU", "Luxembourg", "352", "### ### ###", "628123456", 4, 11, false),
    c("MY", "Malaysia", "60", "##-### ####", "123456789", 9, 10, true),
    c("MX", "Mexico", "52", "## #### ####", "2221234567", 10, 10, false),
    c("MA", "Morocco", "212", "###-######", "650123456", 9, 9, true),
    c("NL", "Netherlands", "31", "# ########", "612345678", 9, 9, true),
    c("NZ", "New Zealand", "64", "## ### ####", "211234567", 8, 10, true),
    c("NG", "Nigeria", "234", "### ### ####", "8021234567", 10, 10, true),
    c("NO", "Norway", "47", "### ## ###", "40612345", 8, 8, false),
    c("PK", "Pakistan", "92", "### #######", "3012345678", 10, 10, true),
    c("PE", "Peru", "51", "### ### ###", "912345678", 9, 9, false),
    c("PH", "Philippines", "63", "### ### ####", "9051234567", 10, 10, true),
    c("PL", "Poland", "48", "### ### ###", "512345678", 9, 9, false),
    c("PT", "Portugal", "351", "### ### ###", "912345678", 9, 9, false),
    c("QA", "Qatar", "974", "#### ####", "33123456", 8, 8, false),
    c("RO", "Romania", "40", "### ### ###", "712034567", 9, 9, true),
    c("RU", "Russia", "7", "### ###-##-##", "9123456789", 10, 10, false),
    c("SA", "Saudi Arabia", "966", "## ### ####", "512345678", 9, 9, true),
    c("RS", "Serbia", "381", "## #######", "601234567", 8, 9, true),
    c("SG", "Singapore", "65", "#### ####", "81234567", 8, 8, false),
    c("SK", "Slovakia", "421", "### ### ###", "912123456", 9, 9, true),
    c("SI", "Slovenia", "386", "## ### ###", "31234567", 8, 8, true),
    c("ZA", "South Africa", "27", "## ### ####", "711234567", 9, 9, true),
    c("KR", "South Korea", "82", "##-####-####", "1020000000", 9, 10, true),
    c("ES", "Spain", "34", "### ## ## ##", "612345678", 9, 9, false),
    c("LK", "Sri Lanka", "94", "## ### ####", "712345678", 9, 9, true),
    c("SE", "Sweden", "46", "##-### ## ##", "701234567", 7, 9, true),
    c("CH", "Switzerland", "41", "## ### ## ##", "781234567", 9, 9, true),
    c("TW", "Taiwan", "886", "### ### ###", "912345678", 9, 9, true),
    c("TH", "Thailand", "66", "## ### ####", "812345678", 9, 9, true),
    c("TR", "Turkey", "90", "### ### ####", "5012345678", 10, 10, true),
    c("UA", "Ukraine", "380", "## ### ####", "501234567", 9, 9, true),
    c("AE", "United Arab Emirates", "971", "## ### ####", "501234567", 8, 9, true),
    c("GB", "United Kingdom", "44", "#### ######", "7400123456", 9, 10, true),
    c("US", "United States", "1", "(###) ###-####", "2015550123", 10, 10, false),
    c("UY", "Uruguay", "598", "## ### ###", "94231234", 8, 8, true),
    c("VN", "Vietnam", "84", "## ### ## ##", "912345678", 9, 9, true),
];

/// Look a country up by ISO code, ignoring case.
#[must_use]
pub fn country(iso: &str) -> Option<&'static Country> {
    COUNTRIES.iter().find(|c| c.iso.eq_ignore_ascii_case(iso))
}

pub(super) fn country_or_first(iso: &str) -> &'static Country {
    country(iso).unwrap_or(&COUNTRIES[0])
}

/// The flag emoji for an ISO code: each letter maps to a regional indicator symbol.
#[must_use]
pub fn flag(iso: &str) -> String {
    iso.chars()
        .filter(char::is_ascii_alphabetic)
        .filter_map(|ch| char::from_u32(0x1F1E6 + u32::from(ch.to_ascii_uppercase()) - u32::from('A')))
        .collect()
}

/// Lay `digits` into `pattern`. Literals only appear once a digit follows them, so
/// partial numbers never end in a dangling separator. Digits past the pattern run on.
#[must_use]
pub fn group(digits: &str, pattern: &str) -> String {
    let mut out = String::new();
    let mut rest = digits.chars().peekable();
    for p in pattern.chars() {
        if rest.peek().is_none() {
            break;
        }
        if p == '#' {
            out.extend(rest.next());
        } else {
            out.push(p);
        }
    }
    out.extend(rest);
    out
}

impl Country {
    /// The number as shown while typing: a typed trunk `0` stays in front.
    #[must_use]
    pub fn display(&self, national: &str) -> String {
        match national.strip_prefix('0').filter(|_| self.trunk) {
            Some(rest) => format!("0{}", group(rest, self.pattern)),
            None => group(national, self.pattern),
        }
    }

    /// The national significant number: digits without the trunk `0`.
    #[must_use]
    pub fn significant<'a>(&self, national: &'a str) -> &'a str {
        if self.trunk { national.strip_prefix('0').unwrap_or(national) } else { national }
    }

    /// The example number laid out, for the placeholder.
    #[must_use]
    pub fn placeholder(&self) -> String {
        group(self.example, self.pattern)
    }

    /// Whether `national` has a plausible length for this country.
    #[must_use]
    pub fn is_valid(&self, national: &str) -> bool {
        let nsn = self.significant(national);
        let len = nsn.len();
        len >= usize::from(self.min) && len <= usize::from(self.max) && !nsn.starts_with('0')
    }

    /// `+` dial code and significant number, or `None` when no digits are typed.
    #[must_use]
    pub fn e164(&self, national: &str) -> Option<String> {
        let nsn = self.significant(national);
        (!nsn.is_empty()).then(|| format!("+{}{nsn}", self.dial))
    }

    /// Most digits the field accepts: the longest number plus a trunk `0`.
    pub(super) fn max_typed(&self) -> usize {
        usize::from(self.max) + usize::from(self.trunk)
    }

    /// Text the picker search matches against.
    fn search_text(&self) -> String {
        format!("{} {} +{}", self.name, self.iso, self.dial)
    }
}

/// Split the digits after a `+` into a country and its number. Longest dial code
/// first; a code shared by several countries (`+1`) keeps `preferred` when it
/// shares it, else the most populous (the US for `+1`). `None` while the digits
/// don't yet name a dial code.
#[must_use]
pub fn split_international(digits: &str, preferred: &str) -> Option<(&'static Country, String)> {
    (1..=3).rev().filter(|n| digits.len() >= *n).find_map(|n| {
        let (dial, rest) = digits.split_at(n);
        let all: Vec<&'static Country> = COUNTRIES.iter().filter(|c| c.dial == dial).collect();
        let pick = all
            .iter()
            .find(|c| c.iso == preferred)
            .or_else(|| all.iter().find(|c| c.iso == "US"))
            .or_else(|| all.first())
            .copied()?;
        Some((pick, rest.to_string()))
    })
}

/// Parse an E.164 string into a country and its national digits.
#[must_use]
pub fn parse_e164(value: &str, preferred: &str) -> Option<(&'static Country, String)> {
    let digits: String = value.trim().strip_prefix('+')?.chars().filter(char::is_ascii_digit).collect();
    split_international(&digits, preferred)
}

/// The countries listed for a search, in display order.
#[must_use]
pub fn visible(query: &str) -> Vec<&'static Country> {
    let needle = listbox::needle(query);
    COUNTRIES.iter().filter(|c| listbox::matches(&c.search_text(), &needle)).collect()
}

/// Whether the country `iso` is listed for a search: [`visible`] for one country.
pub(super) fn is_visible(iso: &str, query: &str) -> bool {
    let needle = listbox::needle(query);
    COUNTRIES.iter().any(|c| c.iso == iso && listbox::matches(&c.search_text(), &needle))
}

pub(super) fn visible_isos(query: &str) -> Vec<&'static str> {
    visible(query).into_iter().map(|c| c.iso).collect()
}
