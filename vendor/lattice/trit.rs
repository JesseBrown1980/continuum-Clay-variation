//! Exact balanced-ternary arithmetic and 27-glyph packing.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(i8)]
pub enum Trit {
    NegativeZero = -1,
    CenterZero = 0,
    PositiveZero = 1,
}

impl Trit {
    pub const ALL: [Self; 3] = [Self::NegativeZero, Self::CenterZero, Self::PositiveZero];

    pub const fn arithmetic_projection(self) -> i8 {
        self as i8
    }

    pub const fn digit(self) -> u8 {
        (self as i8 + 1) as u8
    }

    pub fn from_digit(digit: u8) -> Result<Self, String> {
        match digit {
            0 => Ok(Self::NegativeZero),
            1 => Ok(Self::CenterZero),
            2 => Ok(Self::PositiveZero),
            _ => Err(format!("trit digit {digit} is outside 0..=2")),
        }
    }
}

pub fn balanced_division(value: i64) -> (i64, Trit) {
    let remainder = (value + 1).rem_euclid(3) - 1;
    let quotient = (value - remainder) / 3;
    let trit = match remainder {
        -1 => Trit::NegativeZero,
        0 => Trit::CenterZero,
        1 => Trit::PositiveZero,
        _ => unreachable!("balanced remainder is constrained by rem_euclid"),
    };
    (quotient, trit)
}

pub fn encode_balanced(mut value: i64, width: usize) -> Result<Vec<Trit>, String> {
    let mut digits = Vec::with_capacity(width);
    for _ in 0..width {
        let (quotient, remainder) = balanced_division(value);
        digits.push(remainder);
        value = quotient;
    }
    if value != 0 {
        return Err("value does not fit requested balanced-ternary width".to_string());
    }
    Ok(digits)
}

pub fn decode_balanced(digits_little_endian: &[Trit]) -> Result<i64, String> {
    let mut value = 0_i64;
    for trit in digits_little_endian.iter().rev() {
        value = value
            .checked_mul(3)
            .and_then(|partial| partial.checked_add(i64::from(trit.arithmetic_projection())))
            .ok_or_else(|| "balanced-ternary decode overflow".to_string())?;
    }
    Ok(value)
}

pub const fn pack_trits(trits: [Trit; 3]) -> u8 {
    trits[0].digit() * 9 + trits[1].digit() * 3 + trits[2].digit()
}

pub fn unpack_glyph(glyph: u8) -> Result<[Trit; 3], String> {
    if glyph >= 27 {
        return Err(format!("glyph {glyph} is outside 0..=26"));
    }
    Ok([
        Trit::from_digit(glyph / 9)?,
        Trit::from_digit((glyph / 3) % 3)?,
        Trit::from_digit(glyph % 3)?,
    ])
}

pub fn byte_to_trits(byte: u8) -> [Trit; 6] {
    let digits = encode_balanced(i64::from(byte), 6).expect("every byte fits six trits");
    digits
        .try_into()
        .expect("balanced byte encoder returns six trits")
}

pub fn trits_to_byte(trits: [Trit; 6]) -> Result<u8, String> {
    let value = decode_balanced(&trits)?;
    u8::try_from(value).map_err(|_| format!("balanced value {value} is not one byte"))
}

pub fn byte_to_glyphs(byte: u8) -> [u8; 2] {
    let trits = byte_to_trits(byte);
    [
        pack_trits([trits[0], trits[1], trits[2]]),
        pack_trits([trits[3], trits[4], trits[5]]),
    ]
}

pub fn glyphs_to_byte(glyphs: [u8; 2]) -> Result<u8, String> {
    let low = unpack_glyph(glyphs[0])?;
    let high = unpack_glyph(glyphs[1])?;
    trits_to_byte([low[0], low[1], low[2], high[0], high[1], high[2]])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn balanced_division_is_exact_across_negative_and_positive_values() {
        for value in -100_000_i64..=100_000 {
            let (quotient, remainder) = balanced_division(value);
            assert_eq!(
                value,
                3 * quotient + i64::from(remainder.arithmetic_projection())
            );
            assert!((-1..=1).contains(&remainder.arithmetic_projection()));
        }
    }

    #[test]
    fn every_glyph_is_one_unique_trit_triple() {
        let mut seen = [false; 27];
        for a in Trit::ALL {
            for b in Trit::ALL {
                for c in Trit::ALL {
                    let glyph = pack_trits([a, b, c]);
                    assert!(!seen[usize::from(glyph)]);
                    seen[usize::from(glyph)] = true;
                    assert_eq!(unpack_glyph(glyph).unwrap(), [a, b, c]);
                }
            }
        }
        assert!(seen.into_iter().all(core::convert::identity));
    }

    #[test]
    fn negative_center_and_positive_zero_tags_never_collapse() {
        assert_ne!(Trit::NegativeZero, Trit::CenterZero);
        assert_ne!(Trit::CenterZero, Trit::PositiveZero);
        assert_ne!(Trit::NegativeZero, Trit::PositiveZero);
        assert_eq!(
            Trit::ALL.map(Trit::arithmetic_projection),
            [-1_i8, 0_i8, 1_i8]
        );
        assert_eq!(Trit::ALL.map(Trit::digit), [0_u8, 1_u8, 2_u8]);
    }

    #[test]
    fn all_bytes_survive_b_to_t_to_27_to_t_to_b() {
        for byte in u8::MIN..=u8::MAX {
            let trits = byte_to_trits(byte);
            assert_eq!(trits_to_byte(trits).unwrap(), byte);
            let glyphs = byte_to_glyphs(byte);
            assert!(glyphs.into_iter().all(|glyph| glyph < 27));
            assert_eq!(glyphs_to_byte(glyphs).unwrap(), byte);
        }
    }

    #[test]
    fn width_gate_rejects_unrepresentable_values_and_invalid_glyphs() {
        assert!(encode_balanced(365, 6).is_err());
        assert!(encode_balanced(-365, 6).is_err());
        assert!(unpack_glyph(27).is_err());
    }
}
