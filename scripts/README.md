# LazyMVN Test Scripts

This directory contains test scripts for validating LazyMVN features.

## Test Scripts

### Feature Tests

- **[test_debug_yank.sh](test_debug_yank.sh)** - Test debug yank feature (Shift+Y)
  - Tests comprehensive debug information collection
  - Verifies clipboard functionality
  - Usage: `./scripts/test_debug_yank.sh`

- **[test_yank_logs.sh](test_yank_logs.sh)** - Test log yanking (y)
  - Tests basic output copying
  - Usage: `./scripts/test_yank_logs.sh`

- **[test_yank_logs_guide.sh](test_yank_logs_guide.sh)** - Interactive guide for yank features
  - Step-by-step testing guide
  - Usage: `./scripts/test_yank_logs_guide.sh`

- **[test_yank_logs_integration.sh](test_yank_logs_integration.sh)** - Integration tests for yank
  - Comprehensive yank feature testing
  - Usage: `./scripts/test_yank_logs_integration.sh`

- **[test-profile-loading.sh](test-profile-loading.sh)** - Test profile loading in new tabs
  - Tests Maven profile loading when creating new tabs
  - Validates per-tab profile independence
  - Usage: `./scripts/test-profile-loading.sh`

### System Tests

- **[test-env.sh](test-env.sh)** - Test environment setup
  - Validates Maven and Java installation
  - Usage: `./scripts/test-env.sh`

- **[test-live-reload.sh](test-live-reload.sh)** - Test live reload functionality
  - Tests configuration file watching
  - Tests automatic reloading
  - Usage: `./scripts/test-live-reload.sh`

- **[test-process-cleanup.sh](test-process-cleanup.sh)** - Test process cleanup
  - Tests killing running processes
  - Tests cleanup on exit
  - Usage: `./scripts/test-process-cleanup.sh`

- **[test-starter-isolation.sh](test-starter-isolation.sh)** - Test Spring Boot starter isolation
  - Tests per-tab starter cache
  - Validates no cross-tab interference
  - Usage: `./scripts/test-starter-isolation.sh`

## Running Tests

### Run All Tests
```bash
# From project root
for script in scripts/test*.sh; do
    echo "Running $script..."
    ./"$script"
done
```

### Run Individual Test
```bash
# From project root
./scripts/test_debug_yank.sh
```

### Run from scripts directory
```bash
cd scripts
./test_debug_yank.sh
```

## Test Requirements

- LazyMVN must be built (debug or release)
- Demo projects in `demo/` directory
- Maven or Maven wrapper installed
- Java installed

## Adding New Tests

When adding new test scripts:

1. Make the script executable: `chmod +x scripts/your-test.sh`
2. Use relative paths that work from project root
3. Add clear usage instructions and expected behavior
4. Document the script in this README
5. Verify the script works after being moved to scripts/

## Notes

- Scripts assume they're run from the project root (`/workspaces/lazymvn`)
- Use `--debug` flag to see detailed logging
- Some tests may require manual interaction
