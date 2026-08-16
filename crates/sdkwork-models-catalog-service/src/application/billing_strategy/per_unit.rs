use crate::application::{
    BillingStrategy, BillingStrategyContext, BillingStrategyKind, RateEvaluation,
};
use crate::domain::{BillingMeter, DomainResult};

use super::evaluation::{component, zero_money};
use super::{calculation_mode, measured_quantity, rated_quantity, require_whole_quantity};

#[derive(Debug)]
pub struct TokenUsageBillingStrategy;

impl BillingStrategy for TokenUsageBillingStrategy {
    fn kind(&self) -> BillingStrategyKind {
        BillingStrategyKind::TokenUsage
    }

    fn supports(&self, context: &BillingStrategyContext<'_>) -> bool {
        is_per_unit(context) && is_token_meter(&context.resource.meter)
    }

    fn calculate(&self, context: &BillingStrategyContext<'_>) -> DomainResult<RateEvaluation> {
        calculate_per_unit(context, self.kind(), Some("token"))
    }
}

#[derive(Debug)]
pub struct ApiCallBillingStrategy;

impl BillingStrategy for ApiCallBillingStrategy {
    fn kind(&self) -> BillingStrategyKind {
        BillingStrategyKind::ApiCall
    }

    fn supports(&self, context: &BillingStrategyContext<'_>) -> bool {
        is_per_unit(context) && is_api_call_meter(&context.resource.meter)
    }

    fn calculate(&self, context: &BillingStrategyContext<'_>) -> DomainResult<RateEvaluation> {
        calculate_per_unit(context, self.kind(), Some("API call"))
    }
}

#[derive(Debug)]
pub struct ImageQuantityBillingStrategy;

impl BillingStrategy for ImageQuantityBillingStrategy {
    fn kind(&self) -> BillingStrategyKind {
        BillingStrategyKind::ImageQuantity
    }

    fn supports(&self, context: &BillingStrategyContext<'_>) -> bool {
        is_per_unit(context) && is_image_quantity_meter(&context.resource.meter)
    }

    fn calculate(&self, context: &BillingStrategyContext<'_>) -> DomainResult<RateEvaluation> {
        calculate_per_unit(context, self.kind(), Some("image"))
    }
}

#[derive(Debug)]
pub struct DurationBillingStrategy;

impl BillingStrategy for DurationBillingStrategy {
    fn kind(&self) -> BillingStrategyKind {
        BillingStrategyKind::Duration
    }

    fn supports(&self, context: &BillingStrategyContext<'_>) -> bool {
        is_per_unit(context) && is_duration_meter(&context.resource.meter)
    }

    fn calculate(&self, context: &BillingStrategyContext<'_>) -> DomainResult<RateEvaluation> {
        calculate_per_unit(context, self.kind(), None)
    }
}

#[derive(Debug)]
pub struct UnitQuantityBillingStrategy;

impl BillingStrategy for UnitQuantityBillingStrategy {
    fn kind(&self) -> BillingStrategyKind {
        BillingStrategyKind::UnitQuantity
    }

    fn supports(&self, context: &BillingStrategyContext<'_>) -> bool {
        is_per_unit(context)
            && context.resource.meter != BillingMeter::Unknown
            && !is_token_meter(&context.resource.meter)
            && !is_api_call_meter(&context.resource.meter)
            && !is_image_quantity_meter(&context.resource.meter)
            && !is_duration_meter(&context.resource.meter)
    }

    fn calculate(&self, context: &BillingStrategyContext<'_>) -> DomainResult<RateEvaluation> {
        calculate_per_unit(context, self.kind(), None)
    }
}

fn calculate_per_unit(
    context: &BillingStrategyContext<'_>,
    strategy: BillingStrategyKind,
    whole_quantity_label: Option<&str>,
) -> DomainResult<RateEvaluation> {
    let measured = measured_quantity(context.resource)?;
    if let Some(label) = whole_quantity_label {
        require_whole_quantity(label, measured)?;
    }
    let rated = rated_quantity(context.rate, measured)?;
    let component = component(
        "usage",
        rated,
        context.rate.unit_size,
        context.rate.unit_price.clone(),
        zero_money(&context.rate.unit_price.currency),
    )?;
    RateEvaluation::from_components(strategy, context.rate, measured, rated, vec![component])
}

fn is_per_unit(context: &BillingStrategyContext<'_>) -> bool {
    calculation_mode(context.rate) == "per_unit"
}

fn is_token_meter(meter: &BillingMeter) -> bool {
    matches!(
        meter,
        BillingMeter::LlmInputToken
            | BillingMeter::LlmOutputToken
            | BillingMeter::LlmReasoningToken
            | BillingMeter::LlmCacheWriteToken
            | BillingMeter::LlmCacheReadToken
            | BillingMeter::EmbeddingInputToken
            | BillingMeter::ImageInputToken
            | BillingMeter::ImageOutputToken
            | BillingMeter::AudioInputToken
            | BillingMeter::AudioOutputToken
            | BillingMeter::VideoInputToken
            | BillingMeter::VideoOutputToken
    )
}

fn is_api_call_meter(meter: &BillingMeter) -> bool {
    matches!(
        meter,
        BillingMeter::ApiRequest
            | BillingMeter::ToolCall
            | BillingMeter::WebSearchCall
            | BillingMeter::FileSearchCall
            | BillingMeter::CodeInterpreterSession
            | BillingMeter::ContainerSession
    )
}

fn is_image_quantity_meter(meter: &BillingMeter) -> bool {
    matches!(
        meter,
        BillingMeter::EmbeddingImage
            | BillingMeter::ImageResult
            | BillingMeter::ImagePixel
            | BillingMeter::ImageMegapixel
    )
}

fn is_duration_meter(meter: &BillingMeter) -> bool {
    matches!(
        meter,
        BillingMeter::LlmCacheStorageTokenHour
            | BillingMeter::AudioInputSecond
            | BillingMeter::AudioOutputSecond
            | BillingMeter::AudioInputMinute
            | BillingMeter::AudioOutputMinute
            | BillingMeter::SttAudioMinute
            | BillingMeter::VideoInputSecond
            | BillingMeter::VideoOutputSecond
            | BillingMeter::MusicOutputSecond
            | BillingMeter::StorageGbDay
    )
}
