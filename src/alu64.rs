use crate::i18n::tr;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Files and UI use binary strings exclusively; arithmetic uses exact integers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Word64(pub u64);
impl Serialize for Word64 {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&format!("{:064b}", self.0))
    }
}
impl<'de> Deserialize<'de> for Word64 {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        if s.len() != 64 || !s.bytes().all(|b| b == b'0' || b == b'1') {
            return Err(serde::de::Error::custom(tr(
                "Ожидается строка ровно из 64 двоичных цифр",
            )));
        }
        u64::from_str_radix(&s, 2)
            .map(Self)
            .map_err(serde::de::Error::custom)
    }
}
pub fn parse_word(s: &str) -> Option<Word64> {
    let s: String = s.chars().filter(|c| !c.is_ascii_whitespace()).collect();
    if s.is_empty() || s.len() > 64 || !s.bytes().all(|b| b == b'0' || b == b'1') {
        None
    } else {
        u64::from_str_radix(&s, 2).ok().map(Word64)
    }
}
pub fn binary(word: u64) -> String {
    format!("{word:064b}")
}
pub fn grouped(word: u64) -> String {
    binary(word)
        .as_bytes()
        .chunks(16)
        .map(|row| {
            row.chunks(4)
                .map(|b| std::str::from_utf8(b).unwrap())
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Operation {
    #[serde(rename = "000")]
    Add,
    #[serde(rename = "001")]
    Subtract,
    #[serde(rename = "010")]
    Multiply,
    #[serde(rename = "011")]
    Divide,
    #[serde(rename = "100")]
    Sqrt,
}
impl Operation {
    pub const ALL: [Self; 5] = [
        Self::Add,
        Self::Subtract,
        Self::Multiply,
        Self::Divide,
        Self::Sqrt,
    ];
    pub fn code(self) -> &'static str {
        match self {
            Self::Add => "000",
            Self::Subtract => "001",
            Self::Multiply => "010",
            Self::Divide => "011",
            Self::Sqrt => "100",
        }
    }
    pub fn title(self) -> &'static str {
        match self {
            Self::Add => "A + B",
            Self::Subtract => "A − B",
            Self::Multiply => "A × B",
            Self::Divide => "A / B",
            Self::Sqrt => "√A",
        }
    }
    pub fn description(self) -> &'static str {
        match self {
            Self::Add => tr("Сложение"),
            Self::Subtract => tr("Вычитание"),
            Self::Multiply => tr("Умножение"),
            Self::Divide => tr("Целочисленное деление"),
            Self::Sqrt => tr("Целый квадратный корень"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Alu64 {
    pub a: Word64,
    pub b: Word64,
    pub operation: Operation,
}
impl Default for Alu64 {
    fn default() -> Self {
        Self {
            a: Word64(9),
            b: Word64(3),
            operation: Operation::Add,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Result64 {
    pub low: u64,
    pub high: u64,
    pub remainder: u64,
    pub carry: bool,
    pub borrow: bool,
    pub overflow: bool,
    pub divide_by_zero: bool,
}
impl Alu64 {
    pub fn evaluate(&self) -> Result64 {
        self.evaluate_operation(self.operation)
    }
    pub fn evaluate_operation(&self, op: Operation) -> Result64 {
        let (a, b) = (self.a.0, self.b.0);
        match op {
            Operation::Add => {
                let full = a as u128 + b as u128;
                Result64 {
                    low: full as u64,
                    high: (full >> 64) as u64,
                    carry: full >> 64 != 0,
                    overflow: full >> 64 != 0,
                    ..Default::default()
                }
            }
            Operation::Subtract => {
                let (low, borrow) = a.overflowing_sub(b);
                Result64 {
                    low,
                    borrow,
                    overflow: borrow,
                    ..Default::default()
                }
            }
            Operation::Multiply => {
                let full = a as u128 * b as u128;
                Result64 {
                    low: full as u64,
                    high: (full >> 64) as u64,
                    overflow: full >> 64 != 0,
                    ..Default::default()
                }
            }
            Operation::Divide => {
                if b == 0 {
                    Result64 {
                        divide_by_zero: true,
                        ..Default::default()
                    }
                } else {
                    Result64 {
                        low: a / b,
                        remainder: a % b,
                        ..Default::default()
                    }
                }
            }
            Operation::Sqrt => {
                // Restoring binary square root: two radicand bits per step, no floating point.
                let mut root = 0u64;
                let mut remainder = 0u64;
                for pair in (0..32).rev() {
                    remainder = (remainder << 2) | ((a >> (pair * 2)) & 3);
                    let trial = (root << 2) | 1;
                    root <<= 1;
                    if remainder >= trial {
                        remainder -= trial;
                        root |= 1;
                    }
                }
                Result64 {
                    low: root,
                    remainder,
                    ..Default::default()
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn unit(a: u64, b: u64) -> Alu64 {
        Alu64 {
            a: Word64(a),
            b: Word64(b),
            ..Default::default()
        }
    }
    #[test]
    fn five_operations() {
        let a = unit(9, 3);
        for (op, expected) in [
            (Operation::Add, 12),
            (Operation::Subtract, 6),
            (Operation::Multiply, 27),
            (Operation::Divide, 3),
            (Operation::Sqrt, 3),
        ] {
            assert_eq!(a.evaluate_operation(op).low, expected);
        }
    }
    #[test]
    fn overflow_borrow_and_full_product() {
        let a = unit(u64::MAX, 1).evaluate_operation(Operation::Add);
        assert_eq!((a.low, a.high, a.carry), (0, 1, true));
        let s = unit(0, 1).evaluate_operation(Operation::Subtract);
        assert_eq!(s.low, u64::MAX);
        assert!(s.borrow);
        let m = unit(u64::MAX, u64::MAX).evaluate_operation(Operation::Multiply);
        assert_eq!((m.low, m.high), (1, u64::MAX - 1));
        assert!(m.overflow);
    }
    #[test]
    fn division_remainders_zero_and_high_bits() {
        assert!(
            unit(123, 0)
                .evaluate_operation(Operation::Divide)
                .divide_by_zero
        );
        for (a, b) in [
            (u64::MAX, 7),
            (1 << 63, 3),
            (23, 7),
            (1, 2),
            (0, 1),
            (u64::MAX, u64::MAX),
        ] {
            let r = unit(a, b).evaluate_operation(Operation::Divide);
            assert_eq!(r.low as u128 * b as u128 + r.remainder as u128, a as u128);
            assert!(r.remainder < b);
        }
    }
    #[test]
    fn square_roots_exact_at_boundaries_and_random_inputs() {
        let maxroot = u32::MAX as u64;
        let mut samples = vec![
            0,
            1,
            2,
            3,
            4,
            8,
            9,
            15,
            16,
            17,
            1 << 63,
            u64::MAX,
            maxroot * maxroot,
            maxroot * maxroot - 1,
        ];
        let mut seed = 42u64;
        for _ in 0..2000 {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            samples.push(seed);
        }
        for a in samples {
            let r = unit(a, 0).evaluate_operation(Operation::Sqrt);
            let root = r.low as u128;
            assert!(root * root <= a as u128);
            assert!((root + 1) * (root + 1) > a as u128);
            assert_eq!(root * root + r.remainder as u128, a as u128);
        }
    }
    #[test]
    fn binary_input_and_serialization() {
        assert_eq!(parse_word("0010 0011"), Some(Word64(35)));
        assert!(parse_word("12").is_none());
        assert!(parse_word(&"1".repeat(65)).is_none());
        let a = unit(u64::MAX, 1);
        let s = serde_json::to_string(&a).unwrap();
        assert!(s.contains(&"1".repeat(64)));
        assert_eq!(serde_json::from_str::<Alu64>(&s).unwrap(), a);
        assert!(serde_json::from_str::<Word64>("123").is_err());
        assert!(serde_json::from_str::<Word64>("\"1012\"").is_err());
    }
}
