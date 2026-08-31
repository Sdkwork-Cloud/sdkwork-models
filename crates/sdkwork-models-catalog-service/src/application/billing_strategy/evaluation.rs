use crate::application::{BillingStrategyKind, ResolvedModelPrice, ResolvedPriceSource};
use crate::domain::{DecimalValue, DomainError, DomainResult, ModelPrice, Money};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BillingRateComponent {
    pub component_code: String,
    pub rated_quantity: DecimalValue,
    pub unit_size: DecimalValue,
    pub unit_price: Money,
    pub flat_amount: Money,
    pub amount: Money,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RateEvaluation {
    pub strategy: BillingStrategyKind,
    pub measured_quantity: DecimalValue,
    pub rated_quantity: DecimalValue,
    pub unit_size: DecimalValue,
    pub unit_price: Money,
    pub amount: Money,
    pub charge_timing: String,
    pub calculation_mode: String,
    pub quantity_aggregation: String,
    pub components: Vec<BillingRateComponent>,
}

impl RateEvaluation {
    pub(crate) fn from_components(
        strategy: BillingStrategyKind,
        rate: &ModelPrice,
        measured_quantity: DecimalValue,
        rated_quantity: DecimalValue,
        components: Vec<BillingRateComponent>,
    ) -> DomainResult<Self> {
        let amount = sum_amounts(&rate.unit_price.currency, &components)?;
        let metadata = rate.rate_metadata.as_ref();
        Ok(Self {
            strategy,
            measured_quantity,
            rated_quantity,
            unit_size: rate.unit_size,
            unit_price: rate.unit_price.clone(),
            amount,
            charge_timing: metadata
                .map(|metadata| metadata.charge_timing.clone())
                .unwrap_or_else(|| "usage_reported".to_owned()),
            calculation_mode: metadata
                .map(|metadata| metadata.calculation_mode.clone())
                .unwrap_or_else(|| "per_unit".to_owned()),
            quantity_aggregation: metadata
                .map(|metadata| metadata.quantity_aggregation.clone())
                .unwrap_or_else(|| "sum".to_owned()),
            components,
        })
    }
}

pub(crate) fn component(
    code: impl Into<String>,
    rated_quantity: DecimalValue,
    unit_size: DecimalValue,
    unit_price: Money,
    flat_amount: Money,
) -> DomainResult<BillingRateComponent> {
    if rated_quantity < DecimalValue::ZERO {
        return Err(DomainError::new("billing quantity must not be negative"));
    }
    if unit_size <= DecimalValue::ZERO {
        return Err(DomainError::new("pricing unit size must be positive"));
    }
    if unit_price.unit_price < DecimalValue::ZERO || flat_amount.unit_price < DecimalValue::ZERO {
        return Err(DomainError::new("pricing amounts must not be negative"));
    }
    ensure_currency(&unit_price, &flat_amount)?;
    let usage_amount = unit_price
        .unit_price
        .checked_multiply(rated_quantity)?
        .checked_divide(unit_size)?;
    let amount = Money {
        currency: unit_price.currency.clone(),
        unit_price: usage_amount.checked_add(flat_amount.unit_price)?,
    };
    Ok(BillingRateComponent {
        component_code: code.into(),
        rated_quantity,
        unit_size,
        unit_price,
        flat_amount,
        amount,
    })
}

pub(crate) fn zero_money(currency: &str) -> Money {
    Money {
        currency: currency.to_owned(),
        unit_price: DecimalValue::ZERO,
    }
}

pub(crate) fn scale_evaluation(
    mut evaluation: RateEvaluation,
    multiplier: DecimalValue,
) -> DomainResult<RateEvaluation> {
    if multiplier < DecimalValue::ZERO {
        return Err(DomainError::new("pricing multiplier must not be negative"));
    }
    evaluation.unit_price = evaluation.unit_price.checked_multiply(multiplier)?;
    for component in &mut evaluation.components {
        component.unit_price = component.unit_price.checked_multiply(multiplier)?;
        component.flat_amount = component.flat_amount.checked_multiply(multiplier)?;
        component.amount = component.amount.checked_multiply(multiplier)?;
    }
    evaluation.amount = evaluation.amount.checked_multiply(multiplier)?;
    Ok(evaluation)
}

pub(crate) fn derive_customer_evaluation(
    official: &RateEvaluation,
    price: &ResolvedModelPrice,
) -> DomainResult<RateEvaluation> {
    if price.source != ResolvedPriceSource::DerivedFromOfficialReference {
        return Err(DomainError::new(
            "explicit customer pricing must be evaluated from its own rate",
        ));
    }
    let markup = normalized_markup(price)?;
    let mut components = Vec::with_capacity(official.components.len());
    for official_component in &official.components {
        let unit_price = official_component
            .unit_price
            .checked_multiply(price.reference_multiplier)?
            .add(&markup)?
            .checked_multiply(price.sale_multiplier)?;
        let flat_amount = official_component
            .flat_amount
            .checked_multiply(price.reference_multiplier)?
            .checked_multiply(price.sale_multiplier)?;
        components.push(component(
            official_component.component_code.clone(),
            official_component.rated_quantity,
            official_component.unit_size,
            unit_price,
            flat_amount,
        )?);
    }
    let mut customer = official.clone();
    customer.unit_price = price.customer_charge.clone();
    customer.amount = sum_amounts(&price.customer_charge.currency, &components)?;
    customer.components = components;
    Ok(customer)
}

fn normalized_markup(price: &ResolvedModelPrice) -> DomainResult<Money> {
    if price.default_markup_amount.currency == price.official_reference.unit_price.currency {
        return Ok(price.default_markup_amount.clone());
    }
    if !price.default_markup_amount.is_zero() {
        // A plan markup authored in another currency cannot be added to the
        // official reference; skipping it (with a warning) keeps the derived
        // customer charge usable instead of failing the strategy.
        tracing::warn!(
            charge_currency = %price.official_reference.unit_price.currency,
            markup_currency = %price.default_markup_amount.currency,
            "pricing plan default markup is configured in a different currency than the official reference; the markup is skipped so billing keeps a usable price"
        );
    }
    Ok(zero_money(&price.official_reference.unit_price.currency))
}

fn sum_amounts(currency: &str, components: &[BillingRateComponent]) -> DomainResult<Money> {
    let mut amount = zero_money(currency);
    for component in components {
        ensure_currency(&amount, &component.amount)?;
        amount = amount.add(&component.amount)?;
    }
    Ok(amount)
}

fn ensure_currency(left: &Money, right: &Money) -> DomainResult<()> {
    if left.currency == right.currency {
        Ok(())
    } else {
        Err(DomainError::new("pricing currency mismatch"))
    }
}
