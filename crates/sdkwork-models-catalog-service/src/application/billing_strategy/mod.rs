mod evaluation;
mod flat_fee;
mod formula;
mod per_unit;
mod tiered;

use std::sync::Arc;

use crate::application::ResolvedModelPrice;
use crate::domain::{
    BillingMeter, DecimalValue, DomainError, DomainResult, ModelPrice, Money, PriceSide,
    ResourceDefinition,
};

const RATING_AMOUNT_DECIMAL_PLACES: u32 = 12;

pub use evaluation::{BillingRateComponent, RateEvaluation};
pub use flat_fee::FlatFeeBillingStrategy;
pub use formula::FormulaBillingStrategy;
pub use per_unit::{
    ApiCallBillingStrategy, DurationBillingStrategy, ImageQuantityBillingStrategy,
    TokenUsageBillingStrategy, UnitQuantityBillingStrategy,
};
pub use tiered::{GraduatedTierBillingStrategy, VolumeTierBillingStrategy};

use evaluation::{derive_customer_evaluation, scale_evaluation};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BillingStrategyKind {
    FlatFee,
    TokenUsage,
    ApiCall,
    ImageQuantity,
    Duration,
    UnitQuantity,
    GraduatedTier,
    VolumeTier,
    Formula,
}

impl BillingStrategyKind {
    pub fn code(self) -> &'static str {
        match self {
            Self::FlatFee => "flat_fee",
            Self::TokenUsage => "token_usage",
            Self::ApiCall => "api_call",
            Self::ImageQuantity => "image_quantity",
            Self::Duration => "duration",
            Self::UnitQuantity => "unit_quantity",
            Self::GraduatedTier => "graduated_tier",
            Self::VolumeTier => "volume_tier",
            Self::Formula => "formula",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BillingComponent {
    pub component_code: String,
    pub price_side: PriceSide,
    pub strategy: BillingStrategyKind,
    pub rated_quantity: DecimalValue,
    pub unit_size: DecimalValue,
    pub unit_price: Money,
    pub flat_amount: Money,
    pub amount: Money,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BillingStructure {
    pub strategy: BillingStrategyKind,
    pub meter: BillingMeter,
    pub measured_quantity: DecimalValue,
    pub rated_quantity: DecimalValue,
    pub unit_size: DecimalValue,
    pub official_reference_unit_price: Money,
    pub customer_charge_unit_price: Money,
    pub procurement_cost_unit_price: Option<Money>,
    pub official_reference_amount: Money,
    pub customer_charge_amount: Money,
    pub procurement_cost_amount: Option<Money>,
    pub charge_timing: String,
    pub calculation_mode: String,
    pub quantity_aggregation: String,
    pub components: Vec<BillingComponent>,
}

pub struct BillingStrategyContext<'a> {
    pub resource: &'a ResourceDefinition,
    pub rate: &'a ModelPrice,
}

pub trait BillingStrategy: Send + Sync {
    fn kind(&self) -> BillingStrategyKind;
    fn supports(&self, context: &BillingStrategyContext<'_>) -> bool;
    fn calculate(&self, context: &BillingStrategyContext<'_>) -> DomainResult<RateEvaluation>;
}

#[derive(Clone)]
pub struct BillingStrategyRegistry {
    strategies: Vec<Arc<dyn BillingStrategy>>,
}

impl BillingStrategyRegistry {
    pub fn new(strategies: Vec<Arc<dyn BillingStrategy>>) -> Self {
        Self { strategies }
    }

    pub fn standard() -> Self {
        Self::new(vec![
            Arc::new(FlatFeeBillingStrategy),
            Arc::new(GraduatedTierBillingStrategy),
            Arc::new(VolumeTierBillingStrategy),
            Arc::new(FormulaBillingStrategy),
            Arc::new(TokenUsageBillingStrategy),
            Arc::new(ApiCallBillingStrategy),
            Arc::new(ImageQuantityBillingStrategy),
            Arc::new(DurationBillingStrategy),
            Arc::new(UnitQuantityBillingStrategy),
        ])
    }

    pub fn calculate(
        &self,
        resource: &ResourceDefinition,
        price: &ResolvedModelPrice,
    ) -> DomainResult<BillingStructure> {
        let official = apply_plan_policy(
            self.evaluate(resource, &price.official_reference)?,
            price,
            false,
        )?;
        let customer = match price.raw_customer_charge.as_ref() {
            Some(rate) => apply_plan_policy(
                apply_pricing_rule(
                    scale_evaluation(self.evaluate(resource, rate)?, price.sale_multiplier)?,
                    price,
                )?,
                price,
                true,
            )?,
            None => apply_plan_policy(
                apply_pricing_rule(derive_customer_evaluation(&official, price)?, price)?,
                price,
                true,
            )?,
        };
        let procurement = price
            .raw_upstream_cost
            .as_ref()
            .map(|rate| {
                let multiplier = price.procurement_cost_multiplier.ok_or_else(|| {
                    DomainError::new(
                        "procurement multiplier is required when an upstream rate is present",
                    )
                })?;
                apply_plan_policy(
                    scale_evaluation(self.evaluate(resource, rate)?, multiplier)?,
                    price,
                    false,
                )
            })
            .transpose()?;

        ensure_currency(&official.amount, &customer.amount)?;
        if let Some(procurement) = procurement.as_ref() {
            ensure_currency(&customer.amount, &procurement.amount)?;
        }

        let mut components = billing_components(PriceSide::OfficialReference, &official);
        components.extend(billing_components(PriceSide::CustomerCharge, &customer));
        if let Some(procurement) = procurement.as_ref() {
            components.extend(billing_components(PriceSide::UpstreamCost, procurement));
        }

        Ok(BillingStructure {
            strategy: official.strategy,
            meter: resource.meter.clone(),
            measured_quantity: official.measured_quantity,
            rated_quantity: official.rated_quantity,
            unit_size: official.unit_size,
            official_reference_unit_price: official.unit_price,
            customer_charge_unit_price: customer.unit_price,
            procurement_cost_unit_price: procurement
                .as_ref()
                .map(|evaluation| evaluation.unit_price.clone()),
            official_reference_amount: official.amount,
            customer_charge_amount: customer.amount,
            procurement_cost_amount: procurement.map(|evaluation| evaluation.amount),
            charge_timing: official.charge_timing,
            calculation_mode: official.calculation_mode,
            quantity_aggregation: official.quantity_aggregation,
            components,
        })
    }

    pub fn evaluate(
        &self,
        resource: &ResourceDefinition,
        rate: &ModelPrice,
    ) -> DomainResult<RateEvaluation> {
        let context = BillingStrategyContext { resource, rate };
        let mut selected: Option<&Arc<dyn BillingStrategy>> = None;
        for strategy in &self.strategies {
            if !strategy.supports(&context) {
                continue;
            }
            if let Some(existing) = selected {
                return Err(DomainError::conflict(format!(
                    "billing strategy ambiguous: {} and {} both support meter {}",
                    existing.kind().code(),
                    strategy.kind().code(),
                    resource.meter.code()
                )));
            }
            selected = Some(strategy);
        }
        selected
            .ok_or_else(|| {
                DomainError::not_found(format!(
                    "billing strategy not found for meter {} and calculation mode {}",
                    resource.meter.code(),
                    calculation_mode(rate)
                ))
            })?
            .calculate(&context)
    }
}

fn apply_plan_policy(
    mut evaluation: RateEvaluation,
    price: &ResolvedModelPrice,
    apply_minimum: bool,
) -> DomainResult<RateEvaluation> {
    evaluation.amount.unit_price = evaluation
        .amount
        .unit_price
        .checked_round_to_places(RATING_AMOUNT_DECIMAL_PLACES, &price.rounding_mode)?;
    for component in &mut evaluation.components {
        component.amount.unit_price = component
            .amount
            .unit_price
            .checked_round_to_places(RATING_AMOUNT_DECIMAL_PLACES, &price.rounding_mode)?;
    }
    if apply_minimum {
        if evaluation.amount.currency != price.minimum_charge_amount.currency {
            return Err(DomainError::new("minimum charge currency mismatch"));
        }
        evaluation.amount.unit_price = evaluation
            .amount
            .unit_price
            .max(price.minimum_charge_amount.unit_price);
    }
    Ok(evaluation)
}

fn apply_pricing_rule(
    mut evaluation: RateEvaluation,
    price: &ResolvedModelPrice,
) -> DomainResult<RateEvaluation> {
    let markup = &price.pricing_rule_markup_amount;
    for component in &mut evaluation.components {
        if let Some(unit_price) = price.pricing_rule_unit_price_override.as_ref() {
            component.unit_price = unit_price.clone();
        } else {
            component.unit_price = component
                .unit_price
                .checked_multiply(price.pricing_rule_multiplier)?
                .add(markup)?;
            component.flat_amount = component
                .flat_amount
                .checked_multiply(price.pricing_rule_multiplier)?;
        }
        component.amount = component
            .unit_price
            .unit_price
            .checked_multiply(component.rated_quantity)?
            .checked_divide(component.unit_size)?
            .checked_add(component.flat_amount.unit_price)
            .map(|unit_price| Money {
                currency: component.unit_price.currency.clone(),
                unit_price,
            })?;
    }
    if let Some(unit_price) = price.pricing_rule_unit_price_override.as_ref() {
        evaluation.unit_price = unit_price.clone();
    } else {
        evaluation.unit_price = evaluation
            .unit_price
            .checked_multiply(price.pricing_rule_multiplier)?
            .add(markup)?;
    }
    let mut amount = Money {
        currency: evaluation.amount.currency.clone(),
        unit_price: DecimalValue::ZERO,
    };
    for component in &evaluation.components {
        amount = amount.add(&component.amount)?;
    }
    evaluation.amount = amount;
    Ok(evaluation)
}

impl Default for BillingStrategyRegistry {
    fn default() -> Self {
        Self::standard()
    }
}

fn billing_components(side: PriceSide, evaluation: &RateEvaluation) -> Vec<BillingComponent> {
    evaluation
        .components
        .iter()
        .map(|component| BillingComponent {
            component_code: component.component_code.clone(),
            price_side: side,
            strategy: evaluation.strategy,
            rated_quantity: component.rated_quantity,
            unit_size: component.unit_size,
            unit_price: component.unit_price.clone(),
            flat_amount: component.flat_amount.clone(),
            amount: component.amount.clone(),
        })
        .collect()
}

pub(crate) fn calculation_mode(rate: &ModelPrice) -> &str {
    rate.rate_metadata
        .as_ref()
        .map(|metadata| metadata.calculation_mode.as_str())
        .unwrap_or("per_unit")
}

pub(crate) fn measured_quantity(resource: &ResourceDefinition) -> DomainResult<DecimalValue> {
    resource.measured_quantity.ok_or_else(|| {
        DomainError::new(format!(
            "measured quantity is required for meter {}",
            resource.meter.code()
        ))
    })
}

pub(crate) fn rated_quantity(
    rate: &ModelPrice,
    quantity: DecimalValue,
) -> DomainResult<DecimalValue> {
    let metadata = rate.rate_metadata.as_ref();
    let minimum = metadata
        .map(|metadata| metadata.minimum_quantity)
        .unwrap_or(DecimalValue::ZERO);
    let quantity = quantity.max(minimum);
    match metadata.and_then(|metadata| metadata.quantity_step) {
        Some(step) => quantity.checked_round_up_to_step(step),
        None => Ok(quantity),
    }
}

pub(crate) fn require_whole_quantity(label: &str, quantity: DecimalValue) -> DomainResult<()> {
    let value = quantity.to_fixed_string(12);
    let fraction = value
        .split_once('.')
        .map(|(_, fraction)| fraction)
        .unwrap_or("");
    if fraction.bytes().all(|digit| digit == b'0') {
        Ok(())
    } else {
        Err(DomainError::new(format!(
            "{label} billing quantity must be a whole number"
        )))
    }
}

fn ensure_currency(left: &Money, right: &Money) -> DomainResult<()> {
    if left.currency == right.currency {
        Ok(())
    } else {
        Err(DomainError::new(format!(
            "pricing currency mismatch: {} and {}",
            left.currency, right.currency
        )))
    }
}
