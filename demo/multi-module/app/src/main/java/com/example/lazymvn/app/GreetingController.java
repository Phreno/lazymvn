package com.example.lazymvn.app;

import com.example.lazymvn.library.GreetingService;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.RequestParam;
import org.springframework.web.bind.annotation.RestController;

import java.util.Map;

@RestController
class GreetingController {

    private final GreetingService greetingService;

    GreetingController(GreetingService greetingService) {
        this.greetingService = greetingService;
    }

    @GetMapping("/greet")
    Map<String, String> greet(@RequestParam(value = "name", required = false) String name) {
        return Map.of("message", greetingService.greet(name));
    }
}
