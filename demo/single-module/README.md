# Demo Single Module Project

This is a simple single-module Maven project for demonstrating `lazymvn` with projects that don't have multiple modules.

## Structure

```
single-module/
├── pom.xml                          # Maven project configuration
├── mvnw                             # Maven wrapper script
├── src/
│   ├── main/
│   │   └── java/
│   │       └── com/example/demo/
│   │           ├── App.java         # Main application
│   │           └── Calculator.java  # Sample calculator class
│   └── test/
│       └── java/
│           └── com/example/demo/
│               └── CalculatorTest.java  # Unit tests
└── README.md
```

## Features

- Simple calculator application with basic arithmetic operations
- Unit tests using JUnit 4
- Two Maven profiles: `dev` and `prod`
- Mock Maven wrapper for demonstration

## Usage with lazymvn

From this directory, run:

```bash
lazymvn
```

You should see "(root project)" as the single selectable module, and all Maven commands will work on the entire project.

## Testing the Mock Maven Wrapper

```bash
./mvnw clean compile test
./mvnw package
./mvnw install
./mvnw dependency:tree
```

## Maven Profiles

- `dev` - Development environment profile
- `prod` - Production environment profile

Activate with: `./mvnw -Pdev package`
