# Phase 4: Additional Library Extraction Plan

## Current Status
- ✅ Phase 1-3 Complete: 3 libraries created (1495 lines extracted)
- 📊 Current largest files still in main project

## 📊 File Size Analysis

| File | Lines | Potential for Library |
|------|-------|----------------------|
| `src/maven/command.rs` | 1114 | ⚠️ Already has maven-command-builder |
| `src/maven/detection.rs` | 941 | ✅ HIGH - Maven/Spring detection |
| `src/ui/state/mod.rs` | 836 | ❌ Too UI-specific |
| `src/ui/keybindings/mod.rs` | 646 | ❌ Too UI-specific |
| `src/tui/mod.rs` | 619 | ❌ Too UI-specific |
| `src/ui/state/output.rs` | 568 | ❌ Too UI-specific |
| `src/ui/state/search.rs` | 534 | ❌ Too UI-specific |
| `src/ui/state/profiles.rs` | 503 | ❌ Too UI-specific |
| `src/ui/state/navigation.rs` | 503 | ❌ Too UI-specific |
| `src/core/project.rs` | 475 | ✅ MEDIUM - Project structure |
| `src/core/config/types/preferences.rs` | 472 | ❌ Too specific to app |
| `src/main.rs` | 471 | ❌ Entry point |
| `src/ui/state/commands.rs` | 467 | ❌ Too UI-specific |
| `src/utils/logger.rs` | 451 | ✅ LOW - Generic logging |
| `src/features/history.rs` | 410 | ✅ MEDIUM - Command history |
| `src/features/starters.rs` | 403 | ✅ LOW - Starter templates |
| `src/ui/panes/layout.rs` | 356 | ❌ Too UI-specific |

## 🎯 Phase 4: Recommended Libraries

### Library 1: `maven-project-analyzer` ⭐ **TOP PRIORITY**
**Estimated**: ~900-1000 lines  
**Files to Extract**:
- `src/maven/detection.rs` (941 lines) - Core functionality
- `src/maven/spring.rs` (partial) - Spring detection logic
- `src/core/project.rs` (partial) - POM parsing

**Features**:
- Maven project structure detection
- Multi-module POM.xml parsing
- Spring Boot detection and version analysis
- Maven wrapper (mvnw) detection
- Packaging type detection (jar/war/pom)
- Main class detection
- Plugin detection (spring-boot-maven-plugin, exec-maven-plugin)
- Module relationship mapping
- Parent/child POM relationships

**Benefits**:
- Can be used by other Maven tools
- Standalone project analysis
- IDE integration potential
- CI/CD tooling

**Dependencies**: `quick-xml`, `xmltree`, `serde`

**Estimated Impact**: 
- Main project: -800 lines
- Library: +950 lines (with tests & docs)

---

### Library 2: `maven-command-history` 
**Estimated**: ~400-450 lines  
**Files to Extract**:
- `src/features/history.rs` (410 lines)

**Features**:
- Command history persistence (JSON)
- Project-scoped history
- Timestamp tracking
- Command deduplication
- Frequency-based sorting
- Search and filtering
- History size limits
- Export/import capabilities

**Benefits**:
- Reusable command history system
- Can be used by CLI tools
- Shell integration potential
- Analytics capabilities

**Dependencies**: `serde`, `serde_json`, `chrono`

**Estimated Impact**:
- Main project: -400 lines
- Library: +450 lines (with tests & docs)

---

### Library 3: `maven-pom-parser` (Alternative to #1)
**Estimated**: ~500-600 lines  
**Note**: This would be a subset of `maven-project-analyzer`

**Features**:
- Focused POM.xml parsing
- Dependency extraction
- Plugin configuration parsing
- Property resolution
- Profile parsing
- Inheritance handling

**Decision**: Better to include in `maven-project-analyzer`

---

### Library 4: `maven-starter-templates`
**Estimated**: ~400 lines  
**Files to Extract**:
- `src/features/starters.rs` (403 lines)

**Features**:
- Spring Initializr integration
- Template generation
- Project scaffolding
- Dependency selection
- Archetype support

**Benefits**:
- Project generation tool
- Template management
- Can be CLI tool itself

**Dependencies**: `serde`, `serde_json`, HTTP client

**Estimated Impact**:
- Main project: -400 lines
- Library: +450 lines (with tests & docs)

---

## 📋 Phase 4 Execution Plan

### Stage 1: Maven Project Analyzer (RECOMMENDED FIRST) ⭐

**Priority**: HIGH  
**Complexity**: MEDIUM-HIGH  
**Value**: VERY HIGH

1. **Create library structure**
   ```bash
   mkdir -p crates/maven-project-analyzer/src
   cd crates/maven-project-analyzer
   ```

2. **Extract core detection logic**
   - POM.xml parsing
   - Module detection
   - Spring Boot detection
   - Plugin detection

3. **Add comprehensive tests**
   - Test with real POM files
   - Multi-module scenarios
   - Edge cases

4. **Documentation**
   - API documentation
   - Usage examples
   - Integration guide

**Estimated Time**: 3-4 hours

---

### Stage 2: Maven Command History

**Priority**: MEDIUM  
**Complexity**: LOW  
**Value**: HIGH

1. **Create library structure**
2. **Extract history logic**
3. **Add persistence tests**
4. **Documentation**

**Estimated Time**: 1-2 hours

---

### Stage 3: Maven Starter Templates

**Priority**: LOW  
**Complexity**: MEDIUM  
**Value**: MEDIUM

1. **Create library structure**
2. **Extract template logic**
3. **Add template tests**
4. **Documentation**

**Estimated Time**: 2-3 hours

---

## 🚫 Libraries NOT Recommended

### Why NOT extract UI components?
- **Too coupled**: UI state, keybindings, panes are tightly coupled to LazyMVN's specific UI
- **Not reusable**: Other tools won't share the same UI structure
- **Rapid change**: UI code changes frequently with features
- **Framework-specific**: Tied to ratatui/crossterm

### Why NOT extract logger?
- **Simple**: Only 451 lines, mostly straightforward
- **App-specific**: Session tracking is LazyMVN-specific
- **Many alternatives**: log4rs, env_logger, tracing already exist
- **Low value**: Little reuse potential

---

## 📊 Estimated Total Impact (All Stages)

| Metric | Before | After | Change |
|--------|--------|-------|--------|
| Main project size | ~19,000 lines | ~17,000 lines | -2,000 lines (-10.5%) |
| Libraries | 3 (1495 lines) | 6 (3,345 lines) | +3 libraries |
| Reusable code | 1495 lines | 3,345 lines | +1,850 lines |
| Test coverage | 50 tests | ~90 tests | +40 tests |

---

## ✅ Acceptance Criteria

Each library must have:
- ✅ Comprehensive tests (>80% coverage)
- ✅ Full documentation (README + rustdoc)
- ✅ Examples in examples/ directory
- ✅ No warnings in library code
- ✅ Integration tests in main project
- ✅ Clear, stable API
- ✅ Semantic versioning
- ✅ MIT/Apache-2.0 license
- ✅ Minimal dependencies

---

## 🎯 Recommended Approach for Phase 4

**Start with Stage 1 only**: `maven-project-analyzer`

### Why?
1. **Highest value**: Most reusable component
2. **Clear boundaries**: Well-defined functionality
3. **Significant impact**: ~900 lines extracted
4. **Independent**: Doesn't depend on other libraries
5. **Real use case**: Useful for other Maven tooling

### Hold on Stage 2 & 3:
- Evaluate after Stage 1
- May not be needed if file sizes are manageable
- Consider if planning to publish libraries

---

## 📝 Notes

- **DO NOT PUBLISH**: All libraries remain internal during active development
- **Breaking changes OK**: APIs can change freely
- **Integration first**: Keep main project working at all times
- **Test thoroughly**: Each extraction must maintain all functionality
- **Document later**: Can add comprehensive docs before publishing

---

## 🔄 Alternative: Code Reduction Instead

If library extraction seems excessive, consider:

1. **Refactor large files** into smaller modules
2. **Extract helper functions** into utils
3. **Simplify complex logic** with better patterns
4. **Remove dead code** and unused features
5. **Split UI state** into focused modules

**Target**: Get all files under 600 lines without creating libraries

---

**Next Step**: Decide on approach:
- **A**: Extract `maven-project-analyzer` (recommended)
- **B**: Just refactor large files into smaller modules
- **C**: Do both (extract + refactor)

