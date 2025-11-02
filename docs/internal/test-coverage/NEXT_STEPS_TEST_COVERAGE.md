# Next Steps for Test Coverage Improvement

## Current Status ✅

**Phase 1.5 In Progress**: Adding targeted unit tests for utility modules

- **Test Coverage**: ~8% code coverage (improving)
- **Total Tests**: 820 passing (+463 from baseline)
- **New Test Files**: logger_tests.rs (+17 unit tests)
- **Quality**: All clippy checks passing

## Immediate Priorities

### 1. Logger Module Testing (High Value) ✅ COMPLETE

**File**: `src/utils/logger.rs` (452 lines)  
**Status**: ✅ **17 tests added** in `tests/logger_tests.rs`

**Tests Added**:
- Session extraction (4 tests)
- Last lines reading (4 tests)
- Log level parsing (3 tests)
- Session marker extraction (4 tests)  
- Trace log filtering (2 tests)

**Impact**:
- ✅ +17 unit tests passing
- ✅ Better log parsing reliability
- ✅ Foundation for log analysis features
- ✅ Easier refactoring with test safety net

### 2. State Utilities Testing

**File**: `src/ui/state/utilities.rs` (339 lines, 2 public functions, 0 tests)

**Approach**:
- Identify pure data transformation functions
- Extract from UI-coupled code if needed
- Add focused unit tests

**Expected Impact**:
- +5-8 unit tests
- More reliable state management

### 3. Split Large Files (Deferred)

Files to split once coverage is higher:
- `ui/keybindings/mod.rs` (642 lines) → split into handlers
- `tui/mod.rs` (619 lines) → split event loop, input handling

## Testing Strategy

### For Complex/Coupled Code

When encountering hard-to-test code:

1. **Identify Pure Logic** - What doesn't need I/O?
2. **Extract Helpers** - Move to testable module
3. **Add Tests** - Cover the pure logic first
4. **Refactor Original** - Use the tested helpers

### Pattern Example

```rust
// Before: Hard to test (I/O + logic mixed)
pub fn process_log_file(path: &Path) -> Result<Stats> {
    let content = fs::read_to_string(path)?;
    let mut errors = 0;
    for line in content.lines() {
        if line.contains("ERROR") {
            errors += 1;
        }
    }
    Ok(Stats { errors })
}

// After: Separated and testable
pub fn count_errors(lines: &[&str]) -> usize {  // ← Pure, easy to test
    lines.iter()
        .filter(|line| line.contains("ERROR"))
        .count()
}

pub fn process_log_file(path: &Path) -> Result<Stats> {
    let content = fs::read_to_string(path)?;
    let lines: Vec<&str> = content.lines().collect();
    Ok(Stats { errors: count_errors(&lines) })
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_count_errors() {
        let lines = vec!["INFO: ok", "ERROR: fail", "ERROR: bad"];
        assert_eq!(count_errors(&lines), 2);
    }
}
```

## Coverage Goals

### Short Term (Next 2-3 Sessions)
- [ ] Add tests for logger utilities (+10-15 tests)
- [ ] Add tests for state utilities (+5-8 tests)
- [ ] Reach 40% file coverage (38/95 files)
- [ ] Reach 380+ total tests

### Medium Term
- [ ] Add integration tests for history feature
- [ ] Add integration tests for starters feature
- [ ] Test command execution edge cases
- [ ] Reach 50% file coverage

### Long Term
- [ ] Split files over 600 lines with good test coverage
- [ ] Reach 60% file coverage
- [ ] All critical paths have integration tests
- [ ] Consider adding test coverage reporting tool

## Testing Tools to Consider

### Now
- Continue with built-in `#[test]` - working well
- Keep tests close to code - good for refactoring

### Later (Optional)
- **`cargo-tarpaulin`** - Generate coverage reports (HTML)
- **`cargo-nextest`** - Faster test runner
- **`proptest`** - Property-based testing for parsers
- **`tempfile`** - For testing file operations

## Success Criteria

A module is "well-tested" when:
1. ✅ All pure functions have unit tests
2. ✅ Edge cases are covered (empty, invalid, boundary)
3. ✅ Error paths are tested
4. ✅ Tests are fast (<1ms each)
5. ✅ Tests are deterministic (no flakiness)

## Anti-Patterns to Avoid

❌ **Don't**: Mock everything just to test
✅ **Do**: Extract pure logic and test that

❌ **Don't**: Test implementation details
✅ **Do**: Test behavior and contracts

❌ **Don't**: Write tests for trivial getters/setters
✅ **Do**: Test logic that can have bugs

❌ **Don't**: Make tests depend on each other
✅ **Do**: Keep tests isolated and independent

## Quick Wins Available

1. **XML/Text Parsing** - Already showing good results
2. **String Processing** - Easy to extract and test
3. **Data Transformations** - Pure functions, high value
4. **Validation Logic** - Critical and testable
5. **Format Conversion** - Clear inputs/outputs

## Notes

- Focus on **test quality** over quantity
- Prefer **focused tests** over complex test setups
- Keep **test code simple** - tests shouldn't need tests
- **Refactor for testability** when encountering coupled code
- **Don't break working code** to add tests - extract carefully

---

**Last Updated**: 2025-11-01
**Phase**: 1 of 4 complete
**Next Session**: Logger utilities extraction and testing
