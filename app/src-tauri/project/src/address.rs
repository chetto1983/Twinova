//! IEC 61131-3 direct addresses (`%IX0.0`, `%QD4`, …), and the checks truST leaves to Twinova: it accepts
//! overlapping addresses and a size that does not match the type silently (spec 2026-11-08-component-model-design.md,
//! "Binding to the controller").

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Area {
    Input,
    Output,
    Memory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Size {
    Bit,
    Byte,
    Word,
    Double,
    Long,
}

impl Size {
    pub fn bits(self) -> u64 {
        match self {
            Size::Bit => 1,
            Size::Byte => 8,
            Size::Word => 16,
            Size::Double => 32,
            Size::Long => 64,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Address {
    pub area: Area,
    pub size: Size,
    pub byte: u32,
    /// Only for a bit address.
    pub bit: u8,
}

impl Address {
    pub fn parse(text: &str) -> Result<Self, String> {
        let bad = || format!("'{text}' is not an IEC direct address such as %IX0.0 or %QD4");
        let rest = text.strip_prefix('%').ok_or_else(bad)?;
        let mut chars = rest.chars();
        let area = match chars.next() {
            Some('I') => Area::Input,
            Some('Q') => Area::Output,
            Some('M') => Area::Memory,
            _ => return Err(bad()),
        };
        let size = match chars.next() {
            Some('X') => Size::Bit,
            Some('B') => Size::Byte,
            Some('W') => Size::Word,
            Some('D') => Size::Double,
            Some('L') => Size::Long,
            _ => return Err(bad()),
        };
        let number = chars.as_str();
        let digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
        let (byte, bit) = match (size, number.split_once('.')) {
            (Size::Bit, Some((byte, bit))) if digits(byte) && digits(bit) => (
                byte.parse().map_err(|_| bad())?,
                bit.parse::<u8>().map_err(|_| bad())?,
            ),
            (Size::Bit, _) => return Err(bad()),
            (_, None) if digits(number) => (number.parse().map_err(|_| bad())?, 0),
            _ => return Err(bad()),
        };
        if bit > 7 {
            return Err(format!("'{text}': a bit number goes from 0 to 7"));
        }
        Ok(Self {
            area,
            size,
            byte,
            bit,
        })
    }

    /// The bits it covers in its area, as `[start, end)`.
    pub fn span(self) -> (u64, u64) {
        let start = u64::from(self.byte) * 8 + u64::from(self.bit);
        (start, start + self.size.bits())
    }

    pub fn overlaps(self, other: Address) -> bool {
        let ((a0, a1), (b0, b1)) = (self.span(), other.span());
        self.area == other.area && a0 < b1 && b0 < a1
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let area = match self.area {
            Area::Input => 'I',
            Area::Output => 'Q',
            Area::Memory => 'M',
        };
        let size = match self.size {
            Size::Bit => 'X',
            Size::Byte => 'B',
            Size::Word => 'W',
            Size::Double => 'D',
            Size::Long => 'L',
        };
        match self.size {
            Size::Bit => write!(f, "%{area}{size}{}.{}", self.byte, self.bit),
            _ => write!(f, "%{area}{size}{}", self.byte),
        }
    }
}

#[cfg(test)]
#[path = "address_tests.rs"]
mod tests;
