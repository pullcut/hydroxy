//! Phone field: a country picker and a national number that groups as you type,
//! reported as E.164 (`+447400123456`). Typing or pasting a number that starts
//! with `+` switches to the country its dial code belongs to. Validity is a
//! length check against the country's plan, shown once the field is left.
//!
//! The picker is a searchable list (name, ISO code or dial code) with a virtual
//! highlight, like a combobox.

mod countries;
mod machine;
#[cfg(test)]
mod tests;

pub use countries::{COUNTRIES, Country, country, flag, group, parse_e164, split_international, visible};
pub use machine::{PhoneFieldCommand, PhoneFieldEvent, PhoneFieldInput, PhoneFieldMachine, PhoneFieldState};
