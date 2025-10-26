# Refactoring Priorities Analysis
**Date:** 2025-10-25  
**Objective:** Reduce all files to 200-400 lines per file  
**Current State:** Phase 6 completed, 22 modules created

## Executive Summary

**Files Analysis:**
- 🔴 **9 files > 400 lines** (Priority: HIGH - Need refactoring)
- 🟢 **20+ files in 200-400 range** (Target achieved)
- ✅ **40+ files < 200 lines** (Optimal)

**Total Technical Debt:** ~4,000 excess lines above 400-line threshold

---

## 🔴 Priority 1: Critical Files (> 1000 lines)

### 1. `src/ui/state/mod.rs` - 1,694 lines ⚠️ CRITICAL
**Status:** Already refactored in Phases 1 & 6 (-48% from original 3,255 lines)  
**Analysis:**
- Functions: 69 (coordinator role)
- Structs: 5 (TuiState, ModuleOutput, OutputMetrics, MavenProfile, BuildFlag)
- Impls: 3 (large impl blocks)
- Excess: +1,294 lines above 400

**Current Modules (12 total):**
```
state/
├── mod.rs              1,694 lines ← Coordinator (still large)
├── commands.rs           335 lines
├── output.rs             298 lines
├── project_tab.rs        268 lines
├── search.rs             264 lines
├── navigation.rs         260 lines
├── profiles.rs           203 lines
├── tabs.rs               179 lines
├── config_reload.rs      148 lines
├── launcher_config.rs    120 lines
└── flags.rs               29 lines
```

**Phase 7 Strategy:**
- ✅ **Accept coordinator size**: 69 functions is reasonable for main state orchestration
- ⚠️ **Alternative**: Extract large impl blocks into trait implementations
- 💡 **Complexity analysis**: Check for extractable sections (initialization, validation, UI state)

**Recommendation:** DEFER - File is a coordinator, 1,694 lines acceptable for now. Re-evaluate if new features add complexity.

---

### 2. `src/maven_tests.rs` - 957 lines ⚠️ HIGH PRIORITY
**Analysis:**
- Functions: 33 (all test functions)
- Type: Integration tests file
- Excess: +557 lines above 400

**Strategy:**
Split by test domain into multiple test modules:
```
maven_tests/
├── mod.rs              ~100 lines (shared fixtures)
├── command_tests.rs    ~250 lines (Maven command tests)
├── profile_tests.rs    ~250 lines (Profile loading tests)
├── detection_tests.rs  ~250 lines (Spring Boot detection)
└── settings_tests.rs   ~200 lines (Maven settings tests)
```

**Effort:** 2-3 hours  
**Risk:** LOW (tests are independent)  
**Impact:** HIGH (improves test organization)

---

### 3. `src/core/config.rs` - 773 lines 🎯 MEDIUM-HIGH
**Analysis:**
- Functions: 42
- Structs: 10 (Config, WatchConfig, LoggingConfig, PackageLogLevel, etc.)
- Impls: 4
- Excess: +373 lines above 400

**Domains Identified:**
1. **Config structures** (150 lines): Config, WatchConfig, etc.
2. **Logging config** (200 lines): LoggingConfig, PackageLogLevel, log level parsing
3. **Config loading/saving** (200 lines): File I/O, TOML parsing
4. **Config validation** (150 lines): Validation logic, defaults
5. **Template generation** (73 lines): Already mentioned in code

**Phase 7 Strategy:**
```
core/
├── config/
│   ├── mod.rs           ~150 lines (main Config struct, loading coordination)
│   ├── types.rs         ~150 lines (Config structs: WatchConfig, etc.)
│   ├── logging.rs       ~200 lines (LoggingConfig, PackageLogLevel, validation)
│   ├── template.rs      ~100 lines (Template generation)
│   └── io.rs            ~200 lines (File I/O, TOML parsing/writing)
```

**Effort:** 4-6 hours  
**Risk:** MEDIUM (careful with config loading logic)  
**Impact:** HIGH (config is central to the app)

---

## 🟡 Priority 2: Large Files (600-1000 lines)

### 4. `src/ui/keybindings/mod.rs` - 745 lines
**Analysis:**
- Functions: 17
- Already refactored in Phase 4 (-38% from original 1,203 lines)
- Current submodules: 5 (popup_keys, search_keys, output_keys, command_keys, navigation_keys)
- Excess: +345 lines above 400

**Remaining Content:**
- Main event loop coordination
- Shared key handling logic
- Tab switching logic
- Mode management

**Phase 7 Strategy:**
```
keybindings/
├── mod.rs              ~200 lines (coordination only)
├── tab_keys.rs         ~150 lines (NEW: Tab management keys)
├── mode_manager.rs     ~200 lines (NEW: Mode switching logic)
├── event_router.rs     ~200 lines (NEW: Event routing)
└── [existing modules]  (popup_keys, search_keys, etc.)
```

**Effort:** 3-4 hours  
**Risk:** LOW (event handling is well isolated)  
**Impact:** MEDIUM (improves key handling clarity)

---

### 5. `src/ui/panes/popups.rs` - 646 lines
**Analysis:**
- Functions: 6 (but likely very large functions)
- Domain: Popup rendering (favorites, history, profiles, project selector, etc.)
- Excess: +246 lines above 400

**Phase 7 Strategy:**
Split by popup type:
```
panes/popups/
├── mod.rs              ~100 lines (popup coordination)
├── favorites.rs        ~150 lines (Favorites popup)
├── history.rs          ~150 lines (History popup)
├── profiles.rs         ~150 lines (Profile selection popup)
└── projects.rs         ~150 lines (Project selector popup)
```

**Effort:** 3-4 hours  
**Risk:** LOW (each popup is independent)  
**Impact:** HIGH (popups are complex UI)

---

## 🟢 Priority 3: Medium Files (400-600 lines)

### 6. `src/maven/command.rs` - 556 lines
**Analysis:**
- Functions: 9
- Domain: Maven command building and execution
- Excess: +156 lines above 400

**Potential Split:**
```
maven/
├── command.rs          ~250 lines (main command building)
└── builder.rs          ~300 lines (NEW: Command builder pattern, flags, args)
```

**Effort:** 2-3 hours  
**Risk:** LOW  
**Impact:** MEDIUM

---

### 7. `src/tui/mod.rs` - 540 lines
**Analysis:**
- Functions: 15
- Domain: TUI coordination (created in Phase 5)
- Excess: +140 lines above 400

**Recommendation:** DEFER - Recently created in Phase 5, well-structured, 140 lines excess is acceptable for coordination.

---

### 8. `src/core/project.rs` - 478 lines
**Analysis:**
- Functions: 21
- Domain: POM parsing, module discovery, caching
- Excess: +78 lines above 400

**Potential Split:**
```
core/
├── project.rs          ~250 lines (main module discovery)
└── cache.rs            ~230 lines (NEW: Caching logic, hash validation)
```

**Effort:** 2-3 hours  
**Risk:** MEDIUM (caching is critical)  
**Impact:** MEDIUM

---

### 9. `src/main.rs` - 459 lines
**Analysis:**
- Functions: 3
- Domain: Entry point, CLI parsing, app initialization
- Excess: +59 lines above 400

**Recommendation:** ACCEPTABLE - Entry point files often 400-500 lines. Contains startup logic that benefits from being in one place.

---

## 📊 Refactoring Priority Matrix

| Priority | File | Lines | Excess | Effort | Risk | Impact | Phase |
|----------|------|-------|--------|--------|------|--------|-------|
| 🔴 **P1** | `maven_tests.rs` | 957 | +557 | 2-3h | LOW | HIGH | Phase 7.1 |
| 🔴 **P1** | `core/config.rs` | 773 | +373 | 4-6h | MEDIUM | HIGH | Phase 7.2 |
| 🟡 **P2** | `ui/keybindings/mod.rs` | 745 | +345 | 3-4h | LOW | MEDIUM | Phase 7.3 |
| 🟡 **P2** | `ui/panes/popups.rs` | 646 | +246 | 3-4h | LOW | HIGH | Phase 7.4 |
| 🟢 **P3** | `maven/command.rs` | 556 | +156 | 2-3h | LOW | MEDIUM | Phase 8.1 |
| 🟢 **P3** | `core/project.rs` | 478 | +78 | 2-3h | MEDIUM | MEDIUM | Phase 8.2 |
| ⚪ **DEFER** | `ui/state/mod.rs` | 1,694 | +1,294 | N/A | N/A | N/A | TBD |
| ⚪ **DEFER** | `tui/mod.rs` | 540 | +140 | N/A | N/A | N/A | Recent |
| ⚪ **OK** | `main.rs` | 459 | +59 | N/A | N/A | N/A | Acceptable |

---

## 🎯 Recommended Phase 7 Plan

### Phase 7.1: Split Test File (2-3 hours) ✅ EASIEST
**Target:** `src/maven_tests.rs` (957 → ~100 lines)  
**Action:** Create `tests/` directory with domain-specific test modules  
**Benefit:** Much better test organization, parallel test running potential

### Phase 7.2: Refactor Config Module (4-6 hours) 🎯 HIGH VALUE
**Target:** `src/core/config.rs` (773 → ~150 lines)  
**Action:** Extract logging, I/O, template, validation into submodules  
**Benefit:** Config is central to the app, high reuse potential

### Phase 7.3: Split Keybindings Coordinator (3-4 hours)
**Target:** `src/ui/keybindings/mod.rs` (745 → ~200 lines)  
**Action:** Extract tab keys, mode manager, event router  
**Benefit:** Better separation of concerns for key handling

### Phase 7.4: Split Popup Rendering (3-4 hours)
**Target:** `src/ui/panes/popups.rs` (646 → ~100 lines)  
**Action:** One module per popup type  
**Benefit:** Each popup becomes independently testable

**Total Phase 7 Effort:** ~15-20 hours  
**Total Modules Created:** ~15 new modules  
**Expected Line Reduction:** ~2,500 lines reorganized

---

## 📈 Progress Tracking

### Completed Phases
- ✅ **Phase 1-2**: state/ refactoring (8 modules, -1,366 lines)
- ✅ **Phase 3**: panes/ refactoring (4 modules, -1,295 lines)
- ✅ **Phase 4**: keybindings/ refactoring (5 modules, -458 lines)
- ✅ **Phase 5**: tui/ architectural split (3 modules)
- ✅ **Phase 6**: Micro-refactoring + helpers (2 modules, 23 helpers, -195 lines)

### Phase 7 Goals
- 🎯 Reduce 4 critical files below 400 lines
- 🎯 Create ~15 new modules
- 🎯 Reorganize ~2,500 lines
- 🎯 Maintain 100% test passing (219/219)
- 🎯 Achieve 200-400 line target for 90%+ of codebase

### Success Metrics
- **Before Phase 7:** 9 files > 400 lines
- **After Phase 7:** Target 5 files > 400 lines (ui/state/mod.rs remains as coordinator)
- **File Size Distribution:**
  - 0-200 lines: ~50 files
  - 200-400 lines: ~30 files
  - 400+ lines: <5 files (acceptable coordinators)

---

## 🔍 Next Steps

1. **User Decision:** Choose Phase 7.1, 7.2, 7.3, or 7.4 to start
2. **Pre-refactoring:** Read target file, identify extraction points
3. **Extract:** Create new modules following established patterns
4. **Test:** Ensure 219/219 tests pass
5. **Commit:** Atomic commit with clear documentation
6. **Iterate:** Move to next priority file

**Recommendation:** Start with **Phase 7.1 (maven_tests.rs)** - lowest risk, immediate value, good warm-up for Phase 7.
