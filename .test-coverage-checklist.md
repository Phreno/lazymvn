# Test Coverage Implementation Checklist

## ✅ Completed (2025-10-26)

### Priority 1A: Maven Command Execution ✅
- [x] 19 unit tests for `maven/command.rs`
- [x] 8 integration tests in `tests/command_tests.rs`
- [x] 2 doctests with usage examples
- [x] Coverage: 0% → ~85%

**Tests cover**:
- [x] Command string building (basic, with profiles, modules, flags, settings)
- [x] Root module edge case (omits -pl flag)
- [x] Empty profiles/flags handling
- [x] Argument ordering
- [x] Special characters in paths
- [x] `-f` vs `-pl` flag behavior
- [x] `--also-make` auto-addition for exec:java
- [x] Logging overrides extraction (none/empty/single/multiple)
- [x] Module execution with all options
- [x] Exit code handling
- [x] Complex multi-option scenarios

### Priority 1B: Spring Boot Detection ✅
- [x] 30 unit tests for `maven/detection.rs`
- [x] 2 doctests for platform behavior
- [x] Coverage: ~20% → ~90%

**Tests cover**:
- [x] SpringBootDetection capability checks (9 tests)
  - [x] can_use_spring_boot_run (jar/war/pom packaging)
  - [x] should_prefer_spring_boot_run (war vs jar)
  - [x] can_use_exec_java (plugin/main class)
- [x] Launch strategy decision (6 tests)
  - [x] Force modes (ForceRun, ForceExec)
  - [x] Auto mode decision tree
  - [x] Fallback behavior
- [x] Command building (8 tests)
  - [x] spring-boot:run with profiles/JVM args
  - [x] exec:java with mainClass/JVM args
- [x] XML parsing (7 tests)
  - [x] extract_tag_content with edge cases
  - [x] Whitespace, empty tags, multiple tags
- [x] Platform-specific quoting (Windows/Unix)

### Code Quality ✅
- [x] Export PackageLogLevel from core::config
- [x] All tests passing (57/57)
- [x] Zero compilation warnings
- [x] Platform-specific tests gated properly
- [x] Follows AGENTS.md conventions

### Documentation ✅
- [x] Update TEST_COVERAGE_ANALYSIS.md
- [x] Create TEST_COVERAGE_PROGRESS.md
- [x] Add doctests to functions
- [x] Document test approach

---

## 🎯 Next: Priority 1C (Day 4-5)

### ui/state/commands.rs (~10-12 tests)
- [ ] Command execution state management
  - [ ] test_execute_command_updates_state
  - [ ] test_execute_command_sets_running_flag
  - [ ] test_execute_command_stores_process_info
- [ ] Process lifecycle
  - [ ] test_stop_command_kills_process
  - [ ] test_restart_command_stops_and_starts
  - [ ] test_concurrent_command_prevention
- [ ] Output handling
  - [ ] test_command_output_buffering
  - [ ] test_command_output_streaming
  - [ ] test_command_output_clearing
- [ ] Error states
  - [ ] test_command_failure_state
  - [ ] test_command_timeout_handling

---

## 📊 Progress Tracking

### Week 1: Critical Business Logic
- [x] Day 1-2: maven/command.rs (19 tests) ✅
- [x] Day 3: maven/detection.rs (30 tests) ✅
- [ ] Day 4-5: ui/state/commands.rs (10-12 tests) ⏳

**Target**: 40 tests | **Actual**: 57 tests (+42% over target) 🎉

### Week 2: State Management
- [ ] Day 1-2: ui/state/mod.rs (15-20 tests)
- [ ] Day 3: ui/state/output.rs (8-10 tests)
- [ ] Day 4: ui/state/navigation.rs (8-10 tests)
- [ ] Day 5: ui/state/project_tab.rs (8-10 tests)

**Target**: 50 tests

### Week 3: UI & Utilities
- [ ] Day 1-2: ui/panes/popups.rs (12-15 tests)
- [ ] Day 3: ui/keybindings/popup_keys.rs (8-10 tests)
- [ ] Day 4: utils/logger.rs (5-8 tests)
- [ ] Day 5: utils/loading.rs (3-5 tests)

**Target**: 35 tests

---

## 📈 Metrics

| Milestone | Tests | Status |
|-----------|-------|--------|
| Baseline | 261 | ✅ |
| After Priority 1A+1B | 318 | ✅ Current |
| After Priority 1C | ~330 | 🎯 Next |
| Week 1 Target | ~350 | 🎯 |
| Week 2 Target | ~400 | 🎯 |
| Week 3 Target | ~435 | 🎯 |
| Final Target | ~450 | 🎯 |

**Progress**: 318/450 tests (71% to goal)

---

## 🔍 Test Quality Standards

### ✅ All tests must:
- Follow naming convention: `test_feature_scenario`
- Have clear arrange/act/assert structure
- Test one thing per test
- Include edge cases
- Handle platform differences
- Clean up resources

### ✅ Documentation:
- Add doctests for public API functions
- Include usage examples
- Document edge cases
- Explain platform-specific behavior

### ✅ Organization:
- Unit tests in source `#[cfg(test)]` modules
- Integration tests in `tests/` directory
- Use test fixtures from `tests/common/`
- Platform gates: `#[cfg(unix)]` / `#[cfg(windows)]`

---

## 📝 Commit Template

```
test: add [N] tests for [module name]

[Brief description of what's tested]

Coverage improvements:
- [module]: [before]% → [after]%

Tests added:
- [category 1]: [count] tests
- [category 2]: [count] tests

All tests passing: [passed/total]
```

---

## 🎯 Success Criteria

### Week 1 (Priority 1) ✅ 67% Complete
- [x] Maven command building 100% tested
- [x] Spring Boot detection 100% tested
- [ ] Command execution state 100% tested

### Week 2 (Priority 2)
- [ ] State management core 70% tested
- [ ] Output handling 70% tested
- [ ] Navigation logic 70% tested

### Week 3 (Priority 3)
- [ ] UI components 60% tested
- [ ] Utility functions 80% tested

### Final Goal
- [ ] 450+ total tests
- [ ] 80%+ files >200 lines with tests
- [ ] 95%+ critical modules covered
- [ ] 70%+ logical branches tested

---

Last updated: 2025-10-26
Next update: After Priority 1C completion
