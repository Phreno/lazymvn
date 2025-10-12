# LazyMVN Demo Project - Maven Settings

This demo project includes comprehensive Maven configuration to test LazyMVN's profile and settings functionality.

## Files Added

### 1. `settings.xml`
A comprehensive Maven settings file that includes:

#### Profiles Defined in settings.xml:
- **development** - Development environment with debug enabled
- **production** - Production environment with optimizations
- **testing** - Testing environment with specific configurations  
- **security-scan** - Adds SpotBugs security scanning
- **performance** - Performance testing configuration
- **docker** - Docker build configuration
- **jdk17** - JDK 17 specific settings
- **jdk21** - JDK 21 specific settings

#### Other Settings Features:
- **Local Repository** configuration
- **Plugin Groups** for common plugins
- **Proxy** configuration (commented out)
- **Server** credentials for repositories
- **Mirrors** for Maven Central
- **Repositories** for Spring snapshots and milestones

### 2. Enhanced `pom.xml`
The main pom.xml now includes additional profiles:

#### Profiles Defined in pom.xml:
- **dev** - Auto-activated development profile
- **integration-tests** - Runs integration tests with Failsafe
- **fast** - Fast build that skips tests and checks
- **release** - Production release with source/javadoc jars
- **quality** - Code quality with JaCoCo coverage and Checkstyle
- **db-migrate** - Database migration with Flyway

#### Plus inherited Spring Boot profiles:
- **native** - GraalVM native compilation
- **nativeTest** - Native image testing

### 3. `test-settings.sh`
A helper script that demonstrates various Maven commands using the custom settings.

## How LazyMVN Uses These Files

1. **Automatic Detection**: LazyMVN automatically detects `settings.xml` in the project directory
2. **Profile Loading**: All profiles from both `pom.xml` and `settings.xml` are loaded
3. **Settings Integration**: Maven commands use the settings file automatically
4. **Profile Selection**: Use 'p' key to view and toggle profiles before running commands

## Testing the Setup

### View All Profiles:
```bash
# From demo-project directory
mvn help:all-profiles -N
```

### Run LazyMVN:
```bash
# From demo-project directory  
../target/debug/lazymvn
```

### In LazyMVN:
1. Press `p` to switch to Profiles view
2. Use arrow keys to navigate profiles
3. Press Space to toggle profile selection
4. Press Enter to return to modules
5. Select a module and press Enter to choose a Maven goal
6. Selected profiles will be applied to the Maven command

## Expected Profile Count

You should see approximately **15+ profiles** total:
- ~8 from settings.xml
- ~6 from pom.xml  
- ~3 inherited from Spring Boot parent

## Example Commands

The test script (`test-settings.sh`) shows examples of using the settings:

```bash
# Build with development profile
mvn --settings settings.xml clean compile -Pdev

# Fast build (skip tests)
mvn --settings settings.xml clean package -Pfast

# Release build with quality checks
mvn --settings settings.xml clean package -Prelease,quality
```

## Notes

- The settings.xml file demonstrates many Maven features that can be tested with LazyMVN
- Some profiles include plugins that may not be needed, but serve as examples
- The configuration is designed to be comprehensive for testing purposes
- Real projects would typically have simpler, more focused configurations