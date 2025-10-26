# Multi-Module Spring Boot Demo - Quick Start

This is a real Spring Boot multi-module project for testing LazyMVN keybindings.

## Requirements

- Java 21+
- Maven 3.6+

## Build

**Important:** Always build the project first to ensure dependencies are available:

```bash
mvn clean install
```

This is required because the 's' keybinding runs `mvn -pl app spring-boot:run`, which needs the library module to be already built and installed in the local Maven repository.

## Run with LazyMVN

```bash
# From this directory
lazymvn

# Navigate to 'app' module with arrow keys
# Press 's' to start the Spring Boot application
```

## Test Endpoints

While the app is running (started with `s` keybinding):

```bash
# Default greeting
curl http://localhost:8080/greet
# Returns: {"message":"Howdy from the dev profile!"}

# Custom name
curl http://localhost:8080/greet?name=LazyMVN
# Returns: {"message":"Howdy LazyMVN!"}
```

## Module Structure

- **library** - Shared service (`GreetingService`)
- **app** - Spring Boot web application with REST controller

## Available Profiles

- `dev` (default) - Development settings
- `integration-tests` - Run integration tests
- `fast` - Skip tests and checks
- `release` - Production build
- `quality` - Code quality checks
- `db-migrate` - Database migrations

## Testing LazyMVN Features

1. **Module selection**: Switch between `library` and `app`
2. **Build**: Press `b` to build selected module
3. **Start**: Press `s` on `app` module to start Spring Boot
4. **Profiles**: Press `p` to toggle profiles
5. **Flags**: Press `f` to configure build flags
6. **Tests**: Press `t` to run tests

## Stop Application

Press `Ctrl+C` in LazyMVN to stop the running application.
