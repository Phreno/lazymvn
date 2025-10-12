package com.example.lazymvn.library;

import org.junit.jupiter.api.Test;

import static org.assertj.core.api.Assertions.assertThat;

class GreetingServiceTest {

    private final GreetingService service = new GreetingService("Hello", "from LazyMVN");

    @Test
    void greetsWithFallbackWhenNameMissing() {
        assertThat(service.greet(null)).isEqualTo("Hello from LazyMVN!");
        assertThat(service.greet("   ")).isEqualTo("Hello from LazyMVN!");
    }

    @Test
    void greetsWithProvidedName() {
        assertThat(service.greet("TUI")).isEqualTo("Hello TUI!");
    }
}
