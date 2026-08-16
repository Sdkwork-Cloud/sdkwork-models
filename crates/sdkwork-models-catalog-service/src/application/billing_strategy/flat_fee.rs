use crate::application::{
    BillingStrategy, BillingStrategyContext, BillingStrategyKind, RateEvaluation,
};
use crate::domain::{DecimalValue, DomainError, DomainResult};

use super::evaluation::{component, zero_money};
use super::{calculation_mode, measured_quantity};

#[derive(Debug)]
pub struct FlatFeeBillingStrategy;

impl BillingStrategy for FlatFeeBillingStrategy {
    fn kind(&self) -> BillingStrategyKind {
        BillingStrategyKind::FlatFee
    }

    fn supports(&self, context: &BillingStrategyContext<'_>) -> bool {
        calculation_mode(context.rate) == "flat"
    }

    fn calculate(&self, context: &BillingStrategyContext<'_>) -> DomainResult<RateEvaluation> {
        if context.rate.unit_size != DecimalValue::ONE {
            return Err(DomainError::new(
                "flat fee pricing unit size must equal one",
            ));
        }
        let measured = measured_quantity(context.resource)?;
        let component = component(
            "flat",
            DecimalValue::ONE,
            DecimalValue::ONE,
            context.rate.unit_price.clone(),
            zero_money(&context.rate.unit_price.currency),
        )?;
        RateEvaluation::from_components(
            self.kind(),
            context.rate,
            measured,
            DecimalValue::ONE,
            vec![component],
        )
    }
}
