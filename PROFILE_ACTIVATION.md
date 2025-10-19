# Maven Profile Auto-Activation Handling

## ✅ FULLY IMPLEMENTED (v0.3.0)

LazyMVN now provides **truthful profile activation** that accurately reflects Maven's behavior, including full support for auto-activated profiles and explicit profile deactivation.

## Features

### Three-State Profile System

Each profile can be in one of three states:

1. **Default** - Follows Maven's auto-activation rules
   - Non-auto profiles: inactive (☐)
   - Auto-activated profiles: active (☑ profile (auto))
   - No `-P` argument passed to Maven

2. **Explicitly Enabled** - User manually enabled
   - Shows as: ☑ profile
   - Passes `-P profile` to Maven
   - Overrides Maven's auto-activation

3. **Explicitly Disabled** - User manually disabled
   - Shows as: ☒ profile (disabled)
   - Passes `-P !profile` to Maven
   - Disables even auto-activated profiles

### Auto-Activation Detection

LazyMVN automatically detects which profiles are auto-activated by running `mvn help:active-profiles`. This detects profiles activated by:

- **File existence**: `<activation><file><exists>.project</exists></file></activation>`
- **JDK version**: `<activation><jdk>[11,)</jdk></activation>`
- **Operating System**: `<activation><os><name>Windows</name></os></activation>`
- **Properties**: `<activation><property><name>!skipDev</name></property></activation>`

### Toggle Behavior

Pressing Space/Enter on a profile cycles through states:

**Non-auto-activated profiles:**
```
☐ (Default) → ☑ (Enabled) → ☐ (Default)
```

**Auto-activated profiles:**
```
☑ (auto) (Default) → ☒ (Disabled) → ☑ (auto) (Default)
```

### Visual Indicators

Profiles are color-coded for clarity:

| State | Checkbox | Color | Example |
|-------|----------|-------|---------|
| Default (not auto) | ☐ | White | `☐ prod` |
| Default (auto) | ☑ | Cyan | `☑ dev (auto)` |
| Explicitly Enabled | ☑ | Green | `☑ prod` |
| Explicitly Disabled | ☒ | Red | `☒ dev (disabled)` |

## How It Works

### 1. Profile Discovery

When loading a project:
1. Run `mvn help:all-profiles` to get all available profiles
2. Run `mvn help:active-profiles` to detect auto-activated profiles
3. Create `MavenProfile` structs with correct `auto_activated` flag

### 2. Command Building

When running a Maven command:
1. Collect profile arguments using `to_maven_arg()`:
   - Default state → `None` (no argument)
   - ExplicitlyEnabled → `Some("profile")`
   - ExplicitlyDisabled → `Some("!profile")`
2. Build `-P` argument: `-P profile1,!profile2,profile3`
3. Maven executes with accurate profile configuration

### 3. State Persistence

Module preferences save profile states:
```json
{
  "modules": {
    "library": {
      "active_profiles": ["dev", "!out-eclipse"],
      "enabled_flags": ["-DskipTests"]
    }
  }
}
```

- Explicitly enabled: stored as `"profile"`
- Explicitly disabled: stored as `"!profile"`  
- Default state: not stored (respects auto-activation)

## Example Scenarios

### Scenario 1: Disabling Auto-Activated Profile

**Setup:**
```xml
<profile>
    <id>out-eclipse</id>
    <activation>
        <file><exists>.project</exists></file>
    </activation>
</profile>
```

**.project file exists → Profile auto-activated**

**In LazyMVN:**
1. Profile shows as: `☑ out-eclipse (auto)` (cyan)
2. Press Space: Changes to `☒ out-eclipse (disabled)` (red)
3. Maven command includes: `-P !out-eclipse`
4. Profile is disabled even though .project exists ✅

### Scenario 2: Enabling Non-Auto Profile

**In LazyMVN:**
1. Profile shows as: `☐ prod` (white)
2. Press Space: Changes to `☑ prod` (green)
3. Maven command includes: `-P prod`
4. Profile is explicitly activated ✅

### Scenario 3: Mixed States

**Configuration:**
- `dev` (auto-activated, keeping default)
- `prod` (explicitly enabled)
- `out-eclipse` (auto-activated, explicitly disabled)

**Maven command:**
```bash
mvn clean install -P prod,!out-eclipse
```

**Result:**
- ✅ `dev` active (auto-activation)
- ✅ `prod` active (explicit)
- ✅ `out-eclipse` inactive (explicitly disabled)

## Architecture

### Core Types

```rust
pub enum ProfileState {
    Default,
    ExplicitlyEnabled,
    ExplicitlyDisabled,
}

pub struct MavenProfile {
    pub name: String,
    pub state: ProfileState,
    pub auto_activated: bool,
}

impl MavenProfile {
    pub fn is_active(&self) -> bool;
    pub fn to_maven_arg(&self) -> Option<String>;
    pub fn toggle(&mut self);
}
```

### State in TuiState

```rust
pub struct TuiState {
    pub profiles: Vec<MavenProfile>,  // New
    // Removed: pub active_profiles: Vec<String>
}
```

## Testing

Comprehensive test suite covering:
- ✅ Three-state toggle logic
- ✅ Auto-activation detection
- ✅ Maven argument generation
- ✅ Profile state persistence
- ✅ UI rendering
- ✅ Module switching
- ✅ 75/75 tests passing

## Benefits

1. **Truthful Execution**: UI state exactly matches Maven's behavior
2. **Auto-Profile Support**: Detects and handles auto-activated profiles
3. **Explicit Disable**: Can disable auto-activated profiles with `-P !profile`
4. **Clear UI**: Color-coded visual indicators show profile state
5. **Persistent**: Preferences saved per module, per project
6. **Backward Compatible**: Existing preferences still work

## Migration from v0.2.0

No migration needed! The new system:
- Automatically detects auto-activated profiles
- Interprets old preferences correctly
- Enhances without breaking existing workflows

Old preferences format still works:
```json
{"active_profiles": ["dev", "prod"]}
```

New format supports disable syntax:
```json
{"active_profiles": ["dev", "!out-eclipse", "prod"]}
```

## API Reference

### Functions

```rust
// Get all available profiles
pub fn get_profiles(project_root: &Path) -> Result<Vec<String>, std::io::Error>

// Get auto-activated profiles  
pub fn get_active_profiles(project_root: &Path) -> Result<Vec<String>, std::io::Error>

// Set profiles with auto-activation detection
pub fn set_profiles(&mut self, profile_names: Vec<String>)

// Get active profile names for display
pub fn active_profile_names(&self) -> Vec<String>
```

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `3` | Switch to Profiles view |
| `Space` / `Enter` | Toggle profile state |
| Arrow keys | Navigate profiles |

## Future Enhancements

Potential improvements for future versions:

- [ ] Profile descriptions in tooltip
- [ ] Show activation condition in UI
- [ ] Profile groups/presets
- [ ] Import/export profile configurations
- [ ] Profile activation history

---

## References

- [Maven Profile Activation](https://maven.apache.org/guides/introduction/introduction-to-profiles.html#details-on-profile-activation)
- [Maven Help Plugin](https://maven.apache.org/plugins/maven-help-plugin/)
- [Profile Deactivation Syntax](https://maven.apache.org/guides/introduction/introduction-to-profiles.html#deactivating-a-profile)

---

**Status**: ✅ Fully Implemented in v0.3.0  
**Last Updated**: 2025-10-19  
**Author**: LazyMVN Development Team

