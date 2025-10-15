package com.example.lazymvn.library;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Service;

@Service
public class GreetingService {

    private final String greetingPrefix;
    private final String fallbackName;

    @Autowired
    public GreetingService(
            @Value("${demo.greeting-prefix:Hello}") String greetingPrefix,
            @Value("${demo.default-name:from LazyMVN}") String fallbackName) {
        this.greetingPrefix = greetingPrefix;
        this.fallbackName = fallbackName;
    }

    public String greet(String name) {
        String target = (name == null || name.isBlank()) ? fallbackName : name.trim();
        return greetingPrefix + " " + target + "!";
    }
}
