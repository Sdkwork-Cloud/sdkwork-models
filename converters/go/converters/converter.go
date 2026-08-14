package converters

import (
	"context"

	"github.com/sdkwork-ai/sdkwork-models/converters/go/types"
)

// Converter 转换器接口
type Converter interface {
	// Name 转换器名称
	Name() string

	// SourceProtocol 源协议
	SourceProtocol() types.Protocol

	// TargetProtocol 目标协议
	TargetProtocol() types.Protocol

	// Capabilities 支持的能力
	Capabilities() []types.Capability

	// CanConvert 是否支持该转换
	CanConvert(source, target types.Protocol) bool

	// ConvertRequest 转换请求
	ConvertRequest(ctx context.Context, request types.ConversionRequest) (*types.ConversionRequest, error)

	// ConvertResponse 转换响应
	ConvertResponse(ctx context.Context, response types.ConversionResponse) (*types.ConversionResponse, error)
}

// BaseConverter 基础转换器
type BaseConverter struct {
	name           string
	sourceProtocol types.Protocol
	targetProtocol types.Protocol
	capabilities   []types.Capability
	modelMapping   types.ModelMapping
}

// NewBaseConverter 创建基础转换器
func NewBaseConverter(
	name string,
	sourceProtocol types.Protocol,
	targetProtocol types.Protocol,
	capabilities []types.Capability,
	modelMapping types.ModelMapping,
) *BaseConverter {
	return &BaseConverter{
		name:           name,
		sourceProtocol: sourceProtocol,
		targetProtocol: targetProtocol,
		capabilities:   capabilities,
		modelMapping:   modelMapping,
	}
}

func (c *BaseConverter) Name() string {
	return c.name
}

func (c *BaseConverter) SourceProtocol() types.Protocol {
	return c.sourceProtocol
}

func (c *BaseConverter) TargetProtocol() types.Protocol {
	return c.targetProtocol
}

func (c *BaseConverter) Capabilities() []types.Capability {
	return c.capabilities
}

func (c *BaseConverter) CanConvert(source, target types.Protocol) bool {
	return c.sourceProtocol == source && c.targetProtocol == target
}
