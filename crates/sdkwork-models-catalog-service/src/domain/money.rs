use crate::domain::{DomainError, DomainResult};
use std::ops::Add;

const SCALE: u32 = 12;
const SCALE_FACTOR: i128 = 1_000_000_000_000;

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

    pub fn multiply(self, multiplier: Self) -> Self {
        self.checked_multiply(multiplier)
            .expect("decimal multiplication overflow")
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

    pub fn subtract(self, amount: Self) -> Self {
        self.checked_subtract(amount)
            .expect("decimal subtraction overflow")
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
    type Output = Self;

    fn add(self, amount: Self) -> Self::Output {
        self.checked_add(amount).expect("decimal addition overflow")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Money {
    pub currency: String,
    pub unit_price: DecimalValue,
}

impl Money {
    pub fn new(currency: &str, unit_price: &str) -> DomainResult<Self> {
        let currency = currency.trim().to_ascii_uppercase();
        if currency.len() != 3 || !currency.chars().all(|ch| ch.is_ascii_uppercase()) {
            return Err(DomainError::new(format!(
                "money currency must be a 3-letter ISO 4217 code: {currency}"
            )));
        }
        Ok(Self {
            currency,
            unit_price: DecimalValue::parse(unit_price)?,
        })
    }

    pub fn usd(unit_price: &str) -> DomainResult<Self> {
        Self::new("USD", unit_price)
    }

    pub fn cny(unit_price: &str) -> DomainResult<Self> {
        Self::new("CNY", unit_price)
    }

    pub fn multiply(&self, multiplier: DecimalValue) -> Self {
        self.checked_multiply(multiplier)
            .expect("money multiplication overflow")
    }

    pub fn checked_multiply(&self, multiplier: DecimalValue) -> DomainResult<Self> {
        Ok(Self {
            currency: self.currency.clone(),
            unit_price: self.unit_price.checked_multiply(multiplier)?,
        })
    }

    pub fn add(&self, amount: &Self) -> DomainResult<Self> {
        self.ensure_same_currency(amount)?;
        Ok(Self {
            currency: self.currency.clone(),
            unit_price: self.unit_price.checked_add(amount.unit_price)?,
        })
    }

    pub fn subtract(&self, amount: &Self) -> DomainResult<DecimalValue> {
        self.ensure_same_currency(amount)?;
        self.unit_price.checked_subtract(amount.unit_price)
    }

    pub fn to_fixed_string(&self, digits: u32) -> String {
        self.unit_price.to_fixed_string(digits)
    }

    pub fn is_zero(&self) -> bool {
        self.unit_price.is_zero()
    }

    fn ensure_same_currency(&self, other: &Self) -> DomainResult<()> {
        if self.currency == other.currency {
            Ok(())
        } else {
            Err(DomainError::new("money currency mismatch"))
        }
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
