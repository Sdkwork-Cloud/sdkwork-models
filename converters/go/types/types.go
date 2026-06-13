package types

// Protocol 协议类型
type Protocol string

const (
	ProtocolOpenAIResponses    Protocol = "openai_responses"
	ProtocolOpenAICompletions  Protocol = "openai_completions"
	ProtocolAnthropicMessages  Protocol = "anthropic_messages"
	ProtocolGoogleGemini       Protocol = "google_gemini"
	ProtocolOpenAICompatible   Protocol = "openai_compatible"
	ProtocolVendorNative       Protocol = "vendor_native"
)

// Capability 能力类型
type Capability string

const (
	CapabilityStream    Capability = "stream"
	CapabilityTools     Capability = "tools"
	CapabilityVision    Capability = "vision"
	CapabilityAudio     Capability = "audio"
	CapabilityVideo     Capability = "video"
	CapabilityImage     Capability = "image"
	CapabilityCode      Capability = "code"
	CapabilityReasoning Capability = "reasoning"
)

// Role 消息角色
type Role string

const (
	RoleSystem    Role = "system"
	RoleUser      Role = "user"
	RoleAssistant Role = "assistant"
	RoleTool      Role = "tool"
)

// StopReason 停止原因
type StopReason string

const (
	StopReasonEndTurn      StopReason = "end_turn"
	StopReasonStopSequence StopReason = "stop_sequence"
	StopReasonMaxTokens    StopReason = "max_tokens"
	StopReasonToolUse      StopReason = "tool_use"
	StopReasonStop         StopReason = "stop"
	StopReasonLength       StopReason = "length"
)

// Message 消息
type Message struct {
	Role    Role        `json:"role"`
	Content interface{} `json:"content"` // string 或 []ContentPart
}

// ContentPart 内容部分
type ContentPart struct {
	Type       string      `json:"type"`
	Text       string      `json:"text,omitempty"`
	ImageURL   *ImageURL   `json:"image_url,omitempty"`
	Source     *ImageSource `json:"source,omitempty"`
	ID         string      `json:"id,omitempty"`
	Name       string      `json:"name,omitempty"`
	Input      interface{} `json:"input,omitempty"`
	ToolUseID  string      `json:"tool_use_id,omitempty"`
}

// ImageURL 图片URL
type ImageURL struct {
	URL    string `json:"url"`
	Detail string `json:"detail,omitempty"`
}

// ImageSource 图片源
type ImageSource struct {
	Type      string `json:"type"`
	MediaType string `json:"media_type"`
	Data      string `json:"data"`
}

// Tool 工具定义
type Tool struct {
	Type     string   `json:"type"`
	Function Function `json:"function"`
}

// Function 函数定义
type Function struct {
	Name        string      `json:"name"`
	Description string      `json:"description,omitempty"`
	Parameters  interface{} `json:"parameters,omitempty"`
	InputSchema interface{} `json:"input_schema,omitempty"`
}

// Usage Token使用统计
type Usage struct {
	PromptTokens     int `json:"prompt_tokens"`
	CompletionTokens int `json:"completion_tokens"`
	TotalTokens      int `json:"total_tokens"`
}

// ConversionRequest 转换请求
type ConversionRequest struct {
	Protocol    Protocol    `json:"protocol"`
	Model       string      `json:"model"`
	Messages    []Message   `json:"messages"`
	MaxTokens   *int        `json:"max_tokens,omitempty"`
	Temperature *float64    `json:"temperature,omitempty"`
	TopP        *float64    `json:"top_p,omitempty"`
	Stream      bool        `json:"stream"`
	Tools       []Tool      `json:"tools,omitempty"`
	System      *string     `json:"system,omitempty"`
	Metadata    interface{} `json:"metadata,omitempty"`
}

// ConversionResponse 转换响应
type ConversionResponse struct {
	Protocol   Protocol      `json:"protocol"`
	ID         string        `json:"id"`
	Model      string        `json:"model"`
	Content    []ContentPart `json:"content"`
	StopReason *StopReason   `json:"stop_reason,omitempty"`
	Usage      Usage         `json:"usage"`
	Metadata   interface{}   `json:"metadata,omitempty"`
}

// ModelMapping 模型映射配置
type ModelMapping struct {
	Mapping       map[string]string `json:"mapping"`
	WildcardRules []WildcardRule    `json:"wildcard_rules,omitempty"`
}

// WildcardRule 通配符规则
type WildcardRule struct {
	Pattern string `json:"pattern"`
	Target  string `json:"target"`
}

// Resolve 解析模型名称
func (m *ModelMapping) Resolve(model string) string {
	if target, ok := m.Mapping[model]; ok {
		return target
	}
	return model
}

// ReverseResolve 反向解析模型名称
func (m *ModelMapping) ReverseResolve(model string) string {
	reverse := make(map[string]string)
	for k, v := range m.Mapping {
		reverse[v] = k
	}
	if target, ok := reverse[model]; ok {
		return target
	}
	return model
}
