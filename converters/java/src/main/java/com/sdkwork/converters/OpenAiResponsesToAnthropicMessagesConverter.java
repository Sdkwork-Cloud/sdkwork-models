package com.sdkwork.converters;

import java.util.List;
import java.util.concurrent.CompletableFuture;

/**
 * OPENAI_RESPONSES_TO_ANTHROPIC_MESSAGES 转换器
 */
public class OpenAiResponsesToAnthropicMessagesConverter implements Converter {
    
    private final ModelMapping modelMapping;
    private final ModelMapper mapper;
    
    public OpenAiResponsesToAnthropicMessagesConverter() {
        this(new ModelMapping());
    }
    
    public OpenAiResponsesToAnthropicMessagesConverter(ModelMapping modelMapping) {
        this.modelMapping = modelMapping;
        this.mapper = new ModelMapper();
    }
    
    @Override
    public String name() {
        return "OPENAI_RESPONSES_TO_ANTHROPIC_MESSAGES";
    }
    
    @Override
    public Protocol sourceProtocol() {
        return Protocol.OPENAI_RESPONSES;
    }
    
    @Override
    public Protocol targetProtocol() {
        return Protocol.ANTHROPIC_MESSAGES;
    }
    
    @Override
    public List<Capability> capabilities() {
        return List.of(
            Capability.STREAM,
            Capability.TOOLS,
            Capability.VISION,
            Capability.CODE,
            Capability.REASONING
        );
    }
    
    @Override
    public CompletableFuture<ConversionRequest> convertRequest(ConversionRequest request) {
        return CompletableFuture.supplyAsync(() -> {
            String model = mapper.map(request.getModel(), modelMapping);
            
            List<Message> messages = request.getMessages().stream()
                .filter(m -> m.getRole() != Role.SYSTEM)
                .toList();
            
            return ConversionRequest.builder()
                .protocol(targetProtocol())
                .model(model)
                .messages(messages)
                .maxTokens(request.getMaxTokens())
                .temperature(request.getTemperature())
                .topP(request.getTopP())
                .stream(request.isStream())
                .tools(request.getTools())
                .system(request.getSystem())
                .metadata(request.getMetadata())
                .build();
        });
    }
    
    @Override
    public CompletableFuture<ConversionResponse> convertResponse(ConversionResponse response) {
        return CompletableFuture.supplyAsync(() -> {
            String model = mapper.reverseMap(response.getModel(), modelMapping);
            
            return ConversionResponse.builder()
                .protocol(sourceProtocol())
                .id(response.getId())
                .model(model)
                .content(response.getContent())
                .stopReason(response.getStopReason())
                .usage(response.getUsage())
                .metadata(response.getMetadata())
                .build();
        });
    }
}
