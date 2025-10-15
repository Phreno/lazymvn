# Multi-Module Demo Project

This is a multi-module Maven project for demonstrating LazyMVN with a realistic workspace structure.

## Project Structure

```
multi-module/
├── pom.xml              # Parent aggregator POM
├── library/             # Shared library module
│   ├── pom.xml
│   └── src/
│       ├── main/java/   # GreetingService implementation
│       └── test/java/   # Unit tests
└── app/                 # Spring Boot application
    ├── pom.xml
    └── src/
        ├── main/
        │   ├── java/    # REST controller and main class
        │   └── resources/
        │       ├── application.yml
        │       └── application-dev.yml
        └── test/java/   # Integration tests
```

## Modules

### library
A reusable library module containing:
- `GreetingService` - A Spring bean providing greeting functionality
- Unit tests with JUnit
- No external dependencies beyond Spring Framework

### app
A Spring Boot web application that:
- Depends on the `library` module
- Exposes a REST API at `/greet`
- Supports profile-based configuration (`dev`, `prod`)
- Includes Spring MVC integration tests

## Maven Profiles

This project defines two profiles:

- **dev** - Development profile with verbose logging and development-specific settings
- **prod** - Production profile with optimized settings

## Using with LazyMVN

### 1. Launch LazyMVN
From the repository root:
```bash
lazymvn --project demo/multi-module
```

Or from this directory:
```bash
lazymvn
```

### 2. Module Selection
- Use `↑`/`↓` to select between `library` and `app`
- Press `→` to switch to output pane
- Press `←` to return to module selection

### 3. Try These Commands

**Build the entire project:**
1. Select any module
2. Press `b` (clean install)
3. Watch output in right pane

**Test only the library:**
1. Select `library` module
2. Press `t` (test)
3. Observe test results

**Package the app:**
1. Select `app` module
2. Press `k` (package)
3. Creates executable JAR

**View dependencies:**
1. Select any module
2. Press `d` (dependency:tree)
3. Scroll through dependency graph

### 4. Test Profiles

1. Press `p` to switch to Profiles view
2. Use `↑`/`↓` to select a profile
3. Press `Space` or `Enter` to toggle it
4. Return to modules with `m`
5. Run a command to see profile in action

### 5. Test Build Flags

1. Press `f` to switch to Flags view
2. Toggle flags like "Skip Tests" or "Also Make"
3. Return to modules with `m`
4. Run a command to see flags applied

### 6. Search Output

1. Run a command to generate output
2. Press `/` to start search
3. Type a pattern (e.g., `ERROR`, `BUILD`)
4. Press `Enter` to search
5. Use `n`/`N` to navigate matches

## Manual Maven Commands

For comparison with LazyMVN, try these equivalent commands:

**Build all modules:**
```bash
mvn clean install
```

**Build only library:**
```bash
mvn -pl library clean install
```

**Run app with dev profile:**
```bash
mvn -pl app spring-boot:run -Pdev
```

**Test with skip tests flag:**
```bash
mvn -pl library test -DskipTests
```

**Build library and its dependents:**
```bash
mvn -pl library --also-make-dependents install
```

## Mock Maven Wrapper

This project includes a mock `mvnw` script that simulates Maven output for demonstration purposes. The mock:
- Returns realistic build output
- Simulates different phases (clean, compile, test, package)
- Shows module-specific output
- Responds to profile selections

## Spring Boot Features

The `app` module is a real Spring Boot application. You can run it with:

```bash
mvn -pl app spring-boot:run
```

Then visit:
- `http://localhost:8080/greet?name=LazyMVN` - Default greeting
- `http://localhost:8080/greet?name=LazyMVN` with `-Pdev` - Development greeting

## Maven Settings

This project includes a comprehensive `settings.xml` file demonstrating various Maven settings configurations. LazyMVN automatically detects and uses it when running commands.

**Features demonstrated:**
- Multiple profiles (development, production, testing, security-scan, etc.)
- Local repository configuration
- Plugin groups
- Proxy and mirror settings
- Repository credentials

For detailed information about the Maven settings configuration, see [MAVEN_SETTINGS.md](MAVEN_SETTINGS.md).

## Testing Scenarios

### Scenario 1: Fix a Bug in Library
1. Select `library` module
2. Press `c` to compile
3. Press `t` to run tests
4. Make code changes
5. Press `t` again to verify fix

### Scenario 2: Deploy with Profile
1. Press `p` for profiles view
2. Toggle `prod` profile
3. Press `m` to return to modules
4. Select `app` module
5. Press `i` to install with production settings

### Scenario 3: Dependency Analysis
1. Select `app` module (depends on library)
2. Press `d` for dependency tree
3. Scroll to see library included
4. Press `/` and search for "library"

### Scenario 4: Build with Flags
1. Press `f` for flags view
2. Enable "Skip Tests"
3. Enable "Also Make"
4. Press `m` to return
5. Select `app` module
6. Press `b` to build
7. Observe library is built too (also-make) and tests skipped

## Troubleshooting

### Module not found
If LazyMVN doesn't show both modules:
```bash
grep -A 5 "<modules>" pom.xml
```
Should show `library` and `app`.

### Build fails
Check that you're in the correct directory:
```bash
ls -1
# Should show: pom.xml, library/, app/
```

### Clear cache
If modules don't update after POM changes:
```bash
rm ~/.config/lazymvn/cache.json
```
