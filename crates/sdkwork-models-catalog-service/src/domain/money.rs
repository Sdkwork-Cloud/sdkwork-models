use crate::domain::{DomainError, DomainResult};
pub use sdkwork_models_contract_service::DecimalValue;

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

    pub fn multiply(&self, multiplier: DecimalValue) -> DomainResult<Self> {
        self.checked_multiply(multiplier)
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
