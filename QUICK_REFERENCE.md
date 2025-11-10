# Quick Reference: Functional Refactoring Patterns

## 🎯 Small Function Extraction

```rust
// ❌ BEFORE: Long function
fn process_data(text: &str) -> Option<Result> {
    // 50 lines of mixed concerns...
}

// ✅ AFTER: Composed small functions
fn process_data(text: &str) -> Option<Result> {
    parse(text)
        .and_then(validate)
        .map(transform)
}
```

## 🔀 Option Chaining

```rust
// ❌ BEFORE: Nested if-let
if let Some(x) = try_a() {
    Some(x)
} else if let Some(x) = try_b() {
    Some(x)
} else {
    try_c()
}

// ✅ AFTER: Flat chain
try_a()
    .or_else(try_b)
    .or_else(try_c)
```

## 🔄 Iterator over Loop

```rust
// ❌ BEFORE: Manual loop
let mut results = Vec::new();
for item in items {
    if valid(item) {
        results.push(transform(item));
    }
}

// ✅ AFTER: Iterator chain
let results: Vec<_> = items
    .iter()
    .filter(|x| valid(x))
    .map(transform)
    .collect();
```

## ✅ Predicate Extraction

```rust
// ❌ BEFORE: Inline boolean logic
if text.starts_with("http.") || text.starts_with("https.") || text.starts_with("www.") {
    // ...
}

// ✅ AFTER: Named predicate
fn has_url_prefix(text: &str) -> bool {
    ["http.", "https.", "www."]
        .iter()
        .any(|p| text.starts_with(p))
}

if has_url_prefix(text) {
    // ...
}
```

## 🎨 Naming Conventions

```rust
// Predicates (return bool)
fn is_valid(x: &str) -> bool
fn has_prefix(x: &str) -> bool
fn can_process(x: &Item) -> bool

// Transformations
fn to_lowercase(x: String) -> String
fn parse_input(x: &str) -> Option<Input>

// Try operations
fn try_extract(x: &str) -> Option<Result>
fn try_parse(x: &str) -> Option<Value>
```

## 🧪 Testing Pattern

```rust
#[test]
fn test_function_name() {
    // Happy path
    assert_eq!(func(valid_input), expected);
    
    // Edge cases
    assert!(func("").is_none());
    assert!(func(&"x".repeat(101)).is_none());
    
    // Boundary conditions
    assert!(func(&"x".repeat(100)).is_some());
}
```

## 📏 Function Size Targets

- **Main function**: 3-10 lines
- **Helper functions**: 3-7 lines
- **Complex helpers**: up to 15 lines
- **If > 20 lines**: Consider extracting

## 🚫 Code Smells

```rust
// ❌ Multiple returns
fn check(x: &str) -> bool {
    if cond1 { return true; }
    if cond2 { return true; }
    false
}

// ✅ Single expression
fn check(x: &str) -> bool {
    cond1(x) || cond2(x)
}
```

## 🔗 Common Combinators

```rust
// Option<T>
.map(f)              // Transform value
.and_then(f)         // Chain operations
.or_else(f)          // Try alternative
.filter(p)           // Keep if predicate true
.unwrap_or_default() // Provide default

// Iterator
.iter()              // Create iterator
.filter(p)           // Keep matching items
.map(f)              // Transform items
.filter_map(f)       // Map and filter None
.collect()           // Collect into collection
.any(p)              // Check if any match
.all(p)              // Check if all match
```

## 📋 Refactoring Checklist

- [ ] Function < 20 lines
- [ ] Single responsibility
- [ ] No code duplication
- [ ] Descriptive name
- [ ] Pure (no side effects)
- [ ] Proper types (&str not &String)
- [ ] Tests added
- [ ] Doc comment added
- [ ] Clippy passes

## 🏃‍♂️ Quick Commands

```bash
# Test while developing
cargo watch -x test

# Check quality
cargo clippy

# Format code
cargo fmt

# Run specific test
cargo test test_name -- --nocapture
```

## 💡 Pro Tips

1. **Refactor in small steps** - One function at a time
2. **Run tests after each change** - Catch issues early
3. **Commit frequently** - Small, atomic commits
4. **Use descriptive names** - Code is read more than written
5. **Keep functions pure** - Easier to test and reason about
6. **Prefer composition** - Build complex from simple
7. **Document intent** - Explain the "why", not just the "what"

---

**Remember:** "Any fool can write code that a computer can understand. Good programmers write code that humans can understand." - Martin Fowler
