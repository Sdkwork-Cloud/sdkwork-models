use std::ops::Add;

use crate::{DomainError, DomainResult};

const SCALE: u32 = 12;
const SCALE_FACTOR: i128 = 1_000_000_000_000;

/// Exact signed decimal used by model, pricing, usage, and ranking contracts.
///
/// Values use a fixed scale of twelve decimal places and checked `i128`
/// arithmetic, matching the SDKWork database decimal contract without routing
/// persisted amounts through binary floating point.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DecimalValue {
    scaled: i128,
}

impl DecimalValue {
    pub const ZERO: Self = Self { scaled: 0 };
    pub const ONE: Self = Self {
        scaled: SCALE_FACTOR,
    };

    pub fn parse(value: &str) -> DomainResult<Self> {
        let value = value.trim();
        if value.is_empty() {
            return Err(DomainError::new("decimal value must not be empty"));
        }

        let negative = value.starts_with('-');
        let unsigned = value.trim_start_matches('-');
        let parts: Vec<&str> = unsigned.split('.').collect();
        if parts.len() > 2 || parts[0].is_empty() {
            return Err(DomainError::new(format!("invalid decimal value: {value}")));
        }

        let whole = parse_digits(parts[0], value)?;
        let fraction = if parts.len() == 2 {
            parse_fraction(parts[1], value)?
        } else {
            0
        };
        let scaled = whole
            .checked_mul(SCALE_FACTOR)
            .and_then(|number| number.checked_add(fraction))
            .ok_or_else(|| DomainError::new(format!("decimal value is too large: {value}")))?;

        Ok(Self {
            scaled: if negative { -scaled } else { scaled },
        })
    }

    pub fn multiply(self, multiplier: Self) -> DomainResult<Self> {
        self.checked_multiply(multiplier)
    }

    pub fn checked_multiply(self, multiplier: Self) -> DomainResult<Self> {
        let scaled = self
            .scaled
            .checked_mul(multiplier.scaled)
            .map(|value| value / SCALE_FACTOR)
            .ok_or_else(|| DomainError::new("decimal multiplication overflow"))?;
        decimal_from_scaled(scaled, "decimal multiplication overflow")
    }

    pub fn multiply_i64(self, quantity: i64) -> DomainResult<Self> {
        if quantity < 0 {
            return Err(DomainError::new("decimal quantity must not be negative"));
        }
        let scaled = self
            .scaled
            .checked_mul(quantity as i128)
            .ok_or_else(|| DomainError::new("decimal multiplication overflow"))?;
        decimal_from_scaled(scaled, "decimal multiplication overflow")
    }

    pub fn divide_i64(self, divisor: i64) -> DomainResult<Self> {
        if divisor <= 0 {
            return Err(DomainError::new("decimal divisor must be positive"));
        }
        Ok(Self {
            scaled: self.scaled / divisor as i128,
        })
    }

    pub fn checked_divide(self, divisor: Self) -> DomainResult<Self> {
        if divisor <= Self::ZERO {
            return Err(DomainError::new("decimal divisor must be positive"));
        }
        let scaled = self
            .scaled
            .checked_mul(SCALE_FACTOR)
            .and_then(|value| value.checked_div(divisor.scaled))
            .ok_or_else(|| DomainError::new("decimal division overflow"))?;
        decimal_from_scaled(scaled, "decimal division overflow")
    }

    pub fn subtract(self, amount: Self) -> DomainResult<Self> {
        self.checked_subtract(amount)
    }

    pub fn checked_add(self, amount: Self) -> DomainResult<Self> {
        let scaled = self
            .scaled
            .checked_add(amount.scaled)
            .ok_or_else(|| DomainError::new("decimal addition overflow"))?;
        decimal_from_scaled(scaled, "decimal addition overflow")
    }

    pub fn checked_subtract(self, amount: Self) -> DomainResult<Self> {
        let scaled = self
            .scaled
            .checked_sub(amount.scaled)
            .ok_or_else(|| DomainError::new("decimal subtraction overflow"))?;
        decimal_from_scaled(scaled, "decimal subtraction overflow")
    }

    pub fn max(self, other: Self) -> Self {
        if self >= other {
            self
        } else {
            other
        }
    }

    pub fn checked_round_up_to_step(self, step: Self) -> DomainResult<Self> {
        if self < Self::ZERO {
            return Err(DomainError::new(
                "decimal value must not be negative when rounding to a step",
            ));
        }
        if step <= Self::ZERO {
            return Err(DomainError::new("decimal step must be positive"));
        }
        let remainder = self.scaled % step.scaled;
        if remainder == 0 {
            return Ok(self);
        }
        let increment = step
            .scaled
            .checked_sub(remainder)
            .ok_or_else(|| DomainError::new("decimal step rounding overflow"))?;
        let scaled = self
            .scaled
            .checked_add(increment)
            .ok_or_else(|| DomainError::new("decimal step rounding overflow"))?;
        decimal_from_scaled(scaled, "decimal step rounding overflow")
    }

    pub fn checked_round_to_places(self, places: u32, mode: &str) -> DomainResult<Self> {
        if places > SCALE {
            return Err(DomainError::new(
                "decimal rounding places exceed fixed scale",
            ));
        }
        if !matches!(mode, "half_up" | "half_even" | "up" | "down") {
            return Err(DomainError::new(format!(
                "unsupported decimal rounding mode: {mode}"
            )));
        }
        let factor = 10_i128.pow(SCALE - places);
        let negative = self.scaled < 0;
        let absolute = self.scaled.abs();
        let quotient = absolute / factor;
        let remainder = absolute % factor;
        let increment = match mode {
            "up" => remainder > 0,
            "down" => false,
            "half_up" => remainder.saturating_mul(2) >= factor,
            "half_even" => {
                remainder.saturating_mul(2) > factor
                    || (remainder.saturating_mul(2) == factor && quotient % 2 == 1)
            }
            _ => unreachable!(),
        };
        let rounded = quotient
            .checked_add(i128::from(increment))
            .and_then(|value| value.checked_mul(factor))
            .ok_or_else(|| DomainError::new("decimal rounding overflow"))?;
        decimal_from_scaled(
            if negative { -rounded } else { rounded },
            "decimal rounding overflow",
        )
    }

    pub fn is_zero(self) -> bool {
        self.scaled == 0
    }

    pub fn to_fixed_string(self, digits: u32) -> String {
        assert!(digits <= SCALE);
        let sign = if self.scaled < 0 { "-" } else { "" };
        let absolute = self.scaled.abs();
        let whole = absolute / SCALE_FACTOR;
        let fraction = absolute % SCALE_FACTOR;
        let divisor = 10_i128.pow(SCALE - digits);
        format!(
            "{sign}{whole}.{fraction:0width$}",
            fraction = fraction / divisor,
            width = digits as usize
        )
    }
}

impl Add for DecimalValue {
    type Output = DomainResult<Self>;

    fn add(self, amount: Self) -> Self::Output {
        self.checked_add(amount)
    }
}

fn parse_digits(value: &str, original: &str) -> DomainResult<i128> {
    value
        .parse::<i128>()
        .map_err(|_| DomainError::new(format!("invalid decimal value: {original}")))
}

fn parse_fraction(value: &str, original: &str) -> DomainResult<i128> {
    if value.len() > SCALE as usize || !value.chars().all(|ch| ch.is_ascii_digit()) {
        return Err(DomainError::new(format!(
            "invalid decimal value: {original}"
        )));
    }
    let mut padded = value.to_owned();
    while padded.len() < SCALE as usize {
        padded.push('0');
    }
    parse_digits(&padded, original)
}

fn decimal_from_scaled(scaled: i128, overflow_message: &str) -> DomainResult<DecimalValue> {
    if scaled == i128::MIN {
        return Err(DomainError::new(overflow_message));
    }
    Ok(DecimalValue { scaled })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_addition_preserves_twelve_fraction_digits_beyond_f64_precision() {
        let left = DecimalValue::parse("9007199254740992.000000000001").unwrap();
        let right = DecimalValue::parse("0.000000000009").unwrap();

        assert_eq!(
            left.checked_add(right).unwrap().to_fixed_string(12),
            "9007199254740992.000000000010"
        );
    }

    #[test]
    fn round_up_to_step_preserves_exact_decimal_quantities() {
        let quantity = DecimalValue::parse("5.1").unwrap();
        let step = DecimalValue::parse("0.5").unwrap();

        assert_eq!(
            quantity
                .checked_round_up_to_step(step)
                .unwrap()
                .to_fixed_string(12),
            "5.500000000000"
        );
        assert_eq!(
            DecimalValue::parse("5.0")
                .unwrap()
                .checked_round_up_to_step(step)
                .unwrap()
                .to_fixed_string(12),
            "5.000000000000"
        );
    }
}
