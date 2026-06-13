package com.sdkwork.converters;

import java.util.List;
import java.util.Map;

/**
 * 协议枚举
 */
public enum Protocol {
    OPENAI_RESPONSES("openai_responses"),
    OPENAI_COMPLETIONS("openai_completions"),
    ANTHROPIC_MESSAGES("anthropic_messages"),
    GOOGLE_GEMINI("google_gemini"),
    OPENAI_COMPATIBLE("openai_compatible"),
    VENDOR_NATIVE("vendor_native");

    private final String value;

    Protocol(String value) {
        this.value = value;
    }

    public String getValue() {
        return value;
    }

    public static Protocol fromValue(String value) {
        for (Protocol p : values()) {
            if (p.value.equals(value)) {
                return p;
            }
        }
        throw new IllegalArgumentException("Unknown protocol: " + value);
    }
}
