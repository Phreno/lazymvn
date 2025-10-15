package com.example.demo;

/**
 * Main application entry point.
 */
public class App {
    
    public static void main(String[] args) {
        System.out.println("Demo Single Module Application");
        System.out.println("==============================");
        
        Calculator calc = new Calculator();
        
        System.out.println("2 + 3 = " + calc.add(2, 3));
        System.out.println("10 - 4 = " + calc.subtract(10, 4));
        System.out.println("5 * 6 = " + calc.multiply(5, 6));
        System.out.println("20 / 4 = " + calc.divide(20, 4));
        
        System.out.println("\nApplication completed successfully!");
    }
}
