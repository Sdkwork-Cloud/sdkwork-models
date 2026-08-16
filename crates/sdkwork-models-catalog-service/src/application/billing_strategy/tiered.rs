use crate::application::{
    BillingStrategy, BillingStrategyContext, BillingStrategyKind, RateEvaluation,
};
use crate::domain::{DecimalValue, DomainError, DomainResult, PricingRateTier};

use super::evaluation::component;
use super::{calculation_mode, measured_quantity, rated_quantity};

#[derive(Debug)]
pub struct GraduatedTierBillingStrategy;

impl BillingStrategy for GraduatedTierBillingStrategy {
    fn kind(&self) -> BillingStrategyKind {
        BillingStrategyKind::GraduatedTier
    }

    fn supports(&self, context: &BillingStrategyContext<'_>) -> bool {
        calculation_mode(context.rate) == "graduated"
    }

    fn calculate(&self, context: &BillingStrategyContext<'_>) -> DomainResult<RateEvaluation> {
        let measured = measured_quantity(context.resource)?;
        let rated = rated_quantity(context.rate, measured)?;
        let tiers = validated_tiers(context)?;
        let mut components = Vec::new();
        for tier in tiers {
            if rated <= tier.lower_bound {
                continue;
            }
            let tier_end = tier
                .upper_bound
                .map(|upper| minimum(rated, upper))
                .unwrap_or(rated);
            let tier_quantity = tier_end.checked_subtract(tier.lower_bound)?;
            components.push(component(
                tier.tier_code.clone(),
                tier_quantity,
                tier.unit_size,
                tier.unit_price.clone(),
                tier.flat_amount.clone(),
            )?);
            if tier.upper_bound.is_none_or(|upper| rated < upper) {
                break;
            }
        }
        RateEvaluation::from_components(self.kind(), context.rate, measured, rated, components)
    }
}

#[derive(Debug)]
pub struct VolumeTierBillingStrategy;

impl BillingStrategy for VolumeTierBillingStrategy {
    fn kind(&self) -> BillingStrategyKind {
        BillingStrategyKind::VolumeTier
    }

    fn supports(&self, context: &BillingStrategyContext<'_>) -> bool {
        calculation_mode(context.rate) == "volume"
    }

    fn calculate(&self, context: &BillingStrategyContext<'_>) -> DomainResult<RateEvaluation> {
        let measured = measured_quantity(context.resource)?;
        let rated = rated_quantity(context.rate, measured)?;
        let tiers = validated_tiers(context)?;
        let selected = tiers
            .iter()
            .find(|tier| {
                rated >= tier.lower_bound && tier.upper_bound.is_none_or(|upper| rated < upper)
            })
            .ok_or_else(|| DomainError::new("volume tier not found for rated quantity"))?;
        let component = component(
            selected.tier_code.clone(),
            rated,
            selected.unit_size,
            selected.unit_price.clone(),
            selected.flat_amount.clone(),
        )?;
        RateEvaluation::from_components(self.kind(), context.rate, measured, rated, vec![component])
    }
}

fn validated_tiers<'a>(
    context: &'a BillingStrategyContext<'_>,
) -> DomainResult<&'a [PricingRateTier]> {
    let tiers = &context
        .rate
        .rate_metadata
        .as_ref()
        .ok_or_else(|| DomainError::new("tiered pricing metadata is required"))?
        .tiers;
    if tiers.is_empty() {
        return Err(DomainError::new(
            "tiered pricing requires at least one tier",
        ));
    }
    let mut expected_lower = DecimalValue::ZERO;
    for (index, tier) in tiers.iter().enumerate() {
        if tier.lower_bound != expected_lower {
            return Err(DomainError::new(
                "pricing tiers must start at zero and remain contiguous",
            ));
        }
        if tier.unit_size <= DecimalValue::ZERO {
            return Err(DomainError::new("pricing tier unit size must be positive"));
        }
        match tier.upper_bound {
            Some(upper) if upper <= tier.lower_bound => {
                return Err(DomainError::new(
                    "pricing tier upper bound must be greater than lower bound",
                ));
            }
            Some(upper) => expected_lower = upper,
            None if index + 1 != tiers.len() => {
                return Err(DomainError::new(
                    "only the final pricing tier may be unbounded",
                ));
            }
            None => {}
        }
    }
    if tiers.last().is_some_and(|tier| tier.upper_bound.is_some()) {
        return Err(DomainError::new("final pricing tier must be unbounded"));
    }
    Ok(tiers)
}

fn minimum(left: DecimalValue, right: DecimalValue) -> DecimalValue {
    if left <= right {
        left
    } else {
        right
    }
}
