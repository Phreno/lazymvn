# Functional Refactoring Guidelines

## Philosophy

> "Make the change easy, then make the easy change." - Kent Beck

The goal is to create code that is:
- **Readable**: Intent is clear from function names
- **Testable**: Each function can be tested independently
- **Maintainable**: Changes are isolated to specific functions
- **Composable**: Functions work together naturally
- **Pure**: Same input always produces same output

## When to Refactor

### Good Candidates for Functional Refactoring

✅ **Long Functions** (> 20 lines)
- Break into smaller, focused functions
- Each function should fit on one screen

✅ **Repeated Code Patterns**
- Extract common logic into reusable functions
- DRY (Don't Repeat Yourself)

✅ **Complex Boolean Logic**
- Extract into named predicate functions
- Use `has_*` or `is_*` prefixes

✅ **Nested Conditionals**
- Use early returns or `.and_then()` / `.or_else()`
- Flatten the logic

✅ **Manual Loops**
- Replace with iterator chains
- Use `.map()`, `.filter()`, `.collect()`

✅ **Mutable State**
- Prefer transformations over mutations
- Use functional pipelines

## Refactoring Patterns

### Pattern 1: Extract Small Functions

**Before:**
```rust
fn process(data: &str) -> Option<String> {
    // 50 lines of logic...
}
```

**After:**
```rust
fn process(data: &str) -> Option<String> {
    parse_input(data)
        .and_then(validate)
        .map(transform)
}

fn parse_input(data: &str) -> Option<Input> { /* ... */ }
fn validate(input: Input) -> Option<Input> { /* ... */ }
fn transform(input: Input) -> String { /* ... */ }
```

### Pattern 2: Extract Predicate Functions

**Before:**
```rust
if text.starts_with("http.") || text.starts_with("https.") || text.starts_with("www.") {
    // ...
}
```

**After:**
```rust
if has_url_like_pattern(text) {
    // ...
}

fn has_url_like_pattern(text: &str) -> bool {
    ["http.", "https.", "www."]
        .iter()
        .any(|prefix| text.starts_with(prefix))
}
```

### Pattern 3: Replace Loops with Iterators

**Before:**
```rust
let mut results = Vec::new();
for item in items {
    if let Some(processed) = process(item) {
        if is_valid(&processed) {
            results.push(processed);
        }
    }
}
```

**After:**
```rust
let results: Vec<_> = items
    .iter()
    .filter_map(process)
    .filter(is_valid)
    .collect();
```

### Pattern 4: Chain Option Methods

**Before:**
```rust
if let Some(x) = try_first() {
    return Some(x);
}
if let Some(x) = try_second() {
    return Some(x);
}
try_third()
```

**After:**
```rust
try_first()
    .or_else(try_second)
    .or_else(try_third)
```

### Pattern 5: Use Combinator Methods

**Common Combinators:**
- `.map(f)` - Transform the value if present
- `.and_then(f)` - Chain operations that return Option
- `.or_else(f)` - Try alternative if None
- `.filter(p)` - Keep only if predicate is true
- `.unwrap_or_default()` - Provide default value
- `.ok_or(err)` - Convert Option to Result

## Naming Conventions

### Predicate Functions (return bool)
```rust
fn is_valid(value: &str) -> bool
fn has_prefix(text: &str) -> bool
fn can_process(item: &Item) -> bool
```

### Transformation Functions
```rust
fn to_lowercase(text: String) -> String
fn parse_input(raw: &str) -> Option<Input>
fn collect_results(items: Vec<Item>) -> Vec<Result>
```

### Validation Functions
```rust
fn validate_input(input: &Input) -> Option<Input>
fn check_length(text: &str) -> bool
```

## Code Smells to Avoid

❌ **Long Parameter Lists**
```rust
// Bad
fn process(a: &str, b: &str, c: bool, d: i32, e: Option<String>) -> Result
```

Consider using a struct or builder pattern instead.

❌ **Functions with Side Effects**
```rust
// Bad
fn process(data: &str) -> String {
    println!("Processing: {}", data); // Side effect!
    data.to_uppercase()
}
```

Keep pure functions pure. Log at call site instead.

❌ **Overly Generic Names**
```rust
// Bad
fn handle(data: &str) -> Option<String>
fn do_something(x: i32) -> i32
```

Use descriptive names that indicate purpose.

❌ **Mixing Abstraction Levels**
```rust
// Bad
fn high_level_process(data: &str) -> Result {
    // High level logic
    let byte = data.as_bytes()[0]; // Low level detail
    // More high level logic
}
```

Extract low-level details into separate functions.

## Testing Strategy

### Test Each Function Independently

```rust
#[test]
fn test_is_valid_package_length() {
    assert!(is_valid_package_length("com.example"));
    assert!(is_valid_package_length(&"a".repeat(100)));
    assert!(!is_valid_package_length(""));
    assert!(!is_valid_package_length(&"a".repeat(101)));
}

#[test]
fn test_has_file_extensions() {
    assert!(has_file_extensions("config.xml"));
    assert!(has_file_extensions("data.json"));
    assert!(!has_file_extensions("com.example.Service"));
}
```

### Test Edge Cases

- Empty strings
- Very long strings
- Boundary conditions
- None/Some cases
- Error cases

## Refactoring Workflow

### 1. Add Tests First
Before refactoring, ensure you have tests covering the existing behavior.

### 2. Extract Small Functions
Start by extracting small, pure functions from complex code.

### 3. Run Tests Frequently
After each small change, run tests to ensure nothing broke.

### 4. Improve Names
Give extracted functions clear, descriptive names.

### 5. Add Documentation
Add doc comments explaining what each function does.

### 6. Add Tests for New Functions
Test each extracted function independently.

### 7. Check with Clippy
Run `cargo clippy` to catch common issues.

## Benefits Checklist

After refactoring, you should be able to answer "yes" to:

- ✅ Can I understand each function in under 30 seconds?
- ✅ Can I test each function independently?
- ✅ Are function names descriptive of their purpose?
- ✅ Is the code free of duplication?
- ✅ Does each function have a single responsibility?
- ✅ Can functions be easily composed?
- ✅ Is the code easier to maintain than before?

## Tools and Commands

### Run Tests
```bash
cargo test                    # All tests
cargo test --lib             # Unit tests only
cargo test test_name         # Specific test
```

### Code Quality
```bash
cargo clippy                 # Linting
cargo fmt                    # Formatting
cargo doc --open            # Generate docs
```

### Test Coverage (with tarpaulin)
```bash
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

## Example Refactoring Session

1. **Identify** a complex function
2. **Add tests** if they don't exist
3. **Extract** one small function
4. **Run tests** to verify behavior
5. **Rename** for clarity
6. **Add tests** for extracted function
7. **Repeat** for other parts
8. **Run clippy** to check quality
9. **Commit** changes

## Resources

- **Rust Book**: https://doc.rust-lang.org/book/
- **Rust by Example**: https://doc.rust-lang.org/rust-by-example/
- **Functional Programming in Rust**: https://www.lurklurk.org/effective-rust/
- **Refactoring (Martin Fowler)**: Classic refactoring patterns

## Next Steps

Consider refactoring these areas next:

1. **maven-command-builder**: Command construction logic
2. **maven-log-colorizer**: Color application logic
3. **Integration tests**: Test utilities and helpers

Remember: Refactor incrementally, test frequently, and keep commits small!
