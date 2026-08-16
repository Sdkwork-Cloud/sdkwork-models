use crate::application::{
    BillingStrategy, BillingStrategyContext, BillingStrategyKind, RateEvaluation,
};
use crate::domain::{DecimalValue, DomainError, DomainResult};

use super::evaluation::{component, zero_money};
use super::{calculation_mode, measured_quantity, rated_quantity};

#[derive(Debug)]
pub struct FormulaBillingStrategy;

impl BillingStrategy for FormulaBillingStrategy {
    fn kind(&self) -> BillingStrategyKind {
        BillingStrategyKind::Formula
    }

    fn supports(&self, context: &BillingStrategyContext<'_>) -> bool {
        calculation_mode(context.rate) == "formula"
    }

    fn calculate(&self, context: &BillingStrategyContext<'_>) -> DomainResult<RateEvaluation> {
        let measured = measured_quantity(context.resource)?;
        let formula = context
            .rate
            .rate_metadata
            .as_ref()
            .and_then(|metadata| metadata.formula.as_ref())
            .ok_or_else(|| DomainError::new("formula pricing definition is required"))?;
        let mut units = formula
            .constant_units
            .checked_add(measured.checked_multiply(formula.quantity_coefficient)?)?;
        for term in &formula.terms {
            let value = context
                .resource
                .dimensions
                .decimal(&term.dimension_code)
                .ok_or_else(|| {
                    DomainError::new(format!(
                        "numeric pricing dimension {} is required by formula {}",
                        term.dimension_code, formula.formula_code
                    ))
                })?;
            if value < DecimalValue::ZERO {
                return Err(DomainError::new(format!(
                    "pricing formula dimension {} must not be negative",
                    term.dimension_code
                )));
            }
            units = units.checked_add(value.checked_multiply(term.coefficient)?)?;
        }
        if let Some(minimum) = formula.minimum_units {
            units = units.max(minimum);
        }
        if let Some(maximum) = formula.maximum_units {
            units = minimum(units, maximum);
        }
        let rated = rated_quantity(context.rate, units)?;
        let component = component(
            formula.formula_code.clone(),
            rated,
            context.rate.unit_size,
            context.rate.unit_price.clone(),
            zero_money(&context.rate.unit_price.currency),
        )?;
        RateEvaluation::from_components(self.kind(), context.rate, measured, rated, vec![component])
    }
}

fn minimum(left: DecimalValue, right: DecimalValue) -> DecimalValue {
    if left <= right {
        left
    } else {
        right
    }
}
