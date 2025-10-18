# Maven Profile Auto-Activation Handling

## Current Status

LazyMVN currently lists all available profiles using `mvn help:all-profiles` and allows toggling them via the UI. When profiles are toggled, they are passed to Maven using the `-P profile1,profile2` syntax.

## The Problem

Maven supports **auto-activated profiles** that are automatically enabled based on conditions:
- File existence (`<activation><file><exists>...</exists></file></activation>`)
- JDK version (`<activation><jdk>[11,)</jdk></activation>`)
- Operating system (`<activation><os><name>Windows</name></os></activation>`)
- Properties (`<activation><property><name>!skipDev</name></property></activation>`)

### Current Limitations

1. **Auto-activated profiles are not visible in UI**
   - Users don't know which profiles are auto-activated
   - No way to explicitly disable an auto-activated profile

2. **No support for profile deactivation syntax**
   - Maven supports `-P !profile-name` to explicitly disable a profile
   - LazyMVN doesn't support this syntax

3. **Example Issue**
   ```xml
   <profile>
       <id>out-eclipse</id>
       <activation>
           <file><exists>.project</exists></file>
       </activation>
   </profile>
   ```
   - If `.project` file exists, `out-eclipse` is auto-activated
   - To disable it, you need: `mvn clean install -P !out-eclipse`
   - LazyMVN currently has no way to do this

## Detection Strategy

Maven provides `help:active-profiles` to show currently active profiles:

```bash
$ mvn help:active-profiles

Active Profiles for Project 'com.example:my-app:jar:1.0':

The following profiles are active:

 - dev (source: pom.xml)
 - out-eclipse (source: pom.xml)
```

This shows which profiles are auto-activated in the current environment.

## Proposed Solution

### Phase 1: Detection (Implemented ✓)

Added `get_active_profiles()` function in `src/maven.rs` that:
- Runs `mvn help:active-profiles`
- Parses output to extract auto-activated profiles
- Returns list of profile names

### Phase 2: Enhanced Profile Management (Recommended)

Create a `MavenProfile` struct to track three states:

```rust
pub enum ProfileState {
    Default,              // Follow Maven's auto-activation
    ExplicitlyEnabled,    // Add to -P profile
    ExplicitlyDisabled,   // Add to -P !profile
}

pub struct MavenProfile {
    pub name: String,
    pub state: ProfileState,
    pub auto_activated: bool,  // Detected via help:active-profiles
}
```

### Phase 3: UI Enhancement

Show profile status clearly:
```
Profiles View:
  ☑ dev               (auto-activated)
  ☐ prod              
  ☒ out-eclipse       (disabled, was auto-activated)
  ☑ test              (explicit)
```

Legend:
- `☑` = Active (auto or explicit)
- `☐` = Inactive
- `☒` = Explicitly disabled
- `(auto-activated)` = Would be active by Maven's rules
- `(disabled, was auto-activated)` = User explicitly disabled an auto-activated profile

### Phase 4: Profile Toggle Logic

When user presses Space/Enter on a profile:

| Current State | Auto-Activated | New State | Maven Arg |
|--------------|----------------|-----------|-----------|
| Default | No | ExplicitlyEnabled | `-P profile` |
| Default | Yes | ExplicitlyDisabled | `-P !profile` |
| ExplicitlyEnabled | - | Default | (none) |
| ExplicitlyDisabled | - | Default | (none) |

This provides a three-way toggle:
1. Default (follow Maven)
2. Explicitly enabled (for non-auto profiles)
3. Explicitly disabled (for auto profiles)

## Implementation Complexity

### Current Approach (Simple)
- **Pros**: Works for basic cases, minimal code changes
- **Cons**: No auto-profile detection, no disable support
- **Effort**: Already done

### Enhanced Approach (Comprehensive)
- **Pros**: Full profile control, handles all Maven scenarios
- **Cons**: Requires refactoring ~20 references to `profiles` field
- **Effort**: ~2-3 hours of careful refactoring + testing
- **Risk**: Medium (could break existing functionality if not careful)

## Recommendation

Given that:
1. The current implementation works for manual profile selection
2. Auto-activated profiles are a less common use case
3. Users can work around it by editing `pom.xml` activation conditions
4. A full refactor has moderate risk

**Suggested Approach:**
1. **Document the limitation** in README (done via this file)
2. **Add logging** to show which profiles Maven actually uses
3. **Defer full implementation** to a future release when we can properly test
4. **Provide workaround**: Users needing to disable auto-profiles can:
   - Remove the activation condition from `pom.xml`, OR
   - Use command line: `mvn -P !profile-name ...`

## Testing Auto-Activated Profiles

To test if a profile is auto-activated:

```bash
# Check active profiles
mvn help:active-profiles

# Run with profile explicitly disabled
mvn clean install -P !profile-name
```

## Future Work (v0.4.0+)

- [ ] Implement MavenProfile struct
- [ ] Refactor TuiState to use Vec<MavenProfile>
- [ ] Update UI to show auto-activated status
- [ ] Support three-way toggle (default/enabled/disabled)
- [ ] Add visual indicators (☑/☐/☒)
- [ ] Update module preferences to save ProfileState
- [ ] Add tests for profile state transitions
- [ ] Document in user guide

## Workaround for Users (Current Version)

If you have an auto-activated profile you need to disable:

**Option 1**: Modify POM activation
```xml
<profile>
    <id>out-eclipse</id>
    <!-- Comment out or remove activation -->
    <!--
    <activation>
        <file><exists>.project</exists></file>
    </activation>
    -->
</profile>
```

**Option 2**: Use command line directly
```bash
mvn clean install -P !out-eclipse
```

**Option 3**: Remove trigger file
```bash
rm .project  # If file-based activation
```

## References

- [Maven Profile Activation](https://maven.apache.org/guides/introduction/introduction-to-profiles.html#details-on-profile-activation)
- [Maven Help Plugin](https://maven.apache.org/plugins/maven-help-plugin/)
- [Profile Deactivation Syntax](https://maven.apache.org/guides/introduction/introduction-to-profiles.html#deactivating-a-profile)

---

*This document tracks the profile auto-activation feature status and serves as a design doc for future implementation.*
