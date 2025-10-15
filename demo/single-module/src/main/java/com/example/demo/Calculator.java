package com.example.demo;

/**
 * A simple calculator class for demonstration purposes.
 */
public class Calculator {
    
    /**
     * Adds two numbers together.
     */
    public int add(int a, int b) {
        return a + b;
    }
    
    /**
     * Subtracts b from a.
     */
    public int subtract(int a, int b) {
        return a - b;
    }
    
    /**
     * Multiplies two numbers.
     */
    public int multiply(int a, int b) {
        return a * b;
    }
    
    /**
     * Divides a by b.
     */
    public int divide(int a, int b) {
        if (b == 0) {
            throw new IllegalArgumentException("Division by zero");
        }
        return a / b;
    }
}
