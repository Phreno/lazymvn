# Internal Documentation

This section contains technical documentation for developers and contributors of LazyMVN.

## 📚 Developer Guidelines

- **[AGENTS.md](./AGENTS.md)** - Guidelines for AI agents working on this codebase
- **[VERSIONING.md](./VERSIONING.md)** - Versioning strategy and release process
- **[CONTRIBUTING.md](../../CONTRIBUTING.md)** - How to contribute (see root)

## 🚀 Feature Implementation Guides

### Recent Features (2025)
- **[History Deduplication](./HISTORY_DEDUPLICATION.md)** - Intelligent duplicate command detection and MRU ordering
- **[History Context Switching](./HISTORY_CONTEXT_SWITCHING.md)** - Automatic project switching for history replay
- **[Help Popup Implementation](./HELP_POPUP_IMPLEMENTATION.md)** - Interactive help system (? key)
- **[Ctrl+K Conflict Fix](./FIX_CTRL_K_CONFLICT.md)** - Keybinding disambiguation

### Core Features
- **[Live Reload Implementation](./LIVE_RELOAD_IMPLEMENTATION.md)** - Configuration file watching
- **[Log Formatting Implementation](./LOG_FORMATTING_IMPLEMENTATION.md)** - Log colorization and filtering
- **[Caching Implementation](./CACHING_IMPLEMENTATION.md)** - Maven cache management
- **[Custom Flags Implementation](./CUSTOM_FLAGS_IMPLEMENTATION.md)** - User-defined Maven flags

### Multi-Tab System
- **[Tabs Proposal](./TABS_PROPOSAL.md)** - Original design document
- **[Tabs Implementation](./TABS_IMPLEMENTATION.md)** - Implementation details
- **[Tabs Progress](./TABS_PROGRESS.md)** - Development progress tracking
- **[Tabs Phase 2 Migration](./TABS_PHASE2_MIGRATION.md)** - Migration to improved architecture

## 🐛 Bug Fix Chronicles

### Spring Boot & Logging
- **[Spring Boot Run Fix](./SPRING_BOOT_RUN_FIX.md)** - Launch strategy detection
- **[Spring Boot 1.x Fix Summary](./SPRING_BOOT_1X_FIX_SUMMARY.md)** - JVM args compatibility
- **[Fix Spring Boot 1.x JVM Args](./FIX_SPRING_BOOT_1X_JVM_ARGS.md)** - Detailed implementation
- **[Log Filtering Fix Spring Boot 1.x](./LOG_FILTERING_FIX_SPRING_BOOT_1X.md)** - Log level filtering

### Log4j Debugging Saga
- **[Fix Log4j Success](./FIX_LOG4J_SUCCESS.md)** - Final working solution
- **[Log4j Final Solution](./LOG4J_FINAL_SOLUTION.md)** - Complete implementation
- **[Fix Log4j Filtering](./FIX_LOG4J_FILTERING.md)** - Log level filtering implementation
- **[Fix Log4j Filtering Summary](./FIX_LOG4J_FILTERING_SUMMARY.md)** - Summary
- **[Log4j Fix Complete History](./LOG4J_FIX_COMPLETE_HISTORY.md)** - Full debugging history

#### Log4j Sub-Issues (Historical)
- **[Log4j 1.x Config Fix](./LOG4J_1X_CONFIG_FIX.md)** - Configuration detection
- **[Log4j Agent Implementation](./LOG4J_AGENT_IMPLEMENTATION.md)** - Java agent approach
- **[Log4j Async Bugfix](./LOG4J_ASYNC_BUGFIX.md)** - Async logger issues
- **[Log4j Condition Bug](./LOG4J_CONDITION_BUG.md)** - Conditional logic fix
- **[Log4j Custom Factory Fix](./LOG4J_CUSTOM_FACTORY_FIX.md)** - Factory implementation
- **[Log4j Debug Diagnostic](./LOG4J_DEBUG_DIAGNOSTIC.md)** - Diagnostic tools
- **[Log4j Java Tool Options Fix](./LOG4J_JAVA_TOOL_OPTIONS_FIX.md)** - Environment variables
- **[Log4j Splitn Bug](./LOG4J_SPLITN_BUG.md)** - String parsing fix

### Platform & UI Fixes
- **[Windows Args Fix](./WINDOWS_ARGS_FIX.md)** - PowerShell argument quoting
- **[Test Agent Fix](./TEST_AGENT_FIX.md)** - Java agent testing
- **[Fix Profile Loading New Tab](./FIX_PROFILE_LOADING_NEW_TAB.md)** - Per-tab profile loading
- **[Fix Shared Starter](./FIX_SHARED_STARTER.md)** - Spring Boot starters isolation

## 🔧 Refactoring Documentation

- **[Implementation Summary](./IMPLEMENTATION_SUMMARY.md)** - Launch strategy refactoring
- **[Refactoring](./REFACTORING.md)** - General refactoring guidelines
- **[Refactoring Priorities](./REFACTORING_PRIORITIES.md)** - Priority matrix
- **[Refactoring Summary](./REFACTORING_SUMMARY.md)** - Completed refactorings
- **[Phase 6 - Micro-Refactoring](./PHASE6_MICRO_REFACTORING.md)** - Small improvements
- **[Phase 7.1 - Plan](./PHASE_7.1_PLAN.md)** - Future planning

## 🧪 Testing

- **[Test Coverage Analysis](./TEST_COVERAGE_ANALYSIS.md)** - Coverage reports
- **[Test Coverage Progress](./TEST_COVERAGE_PROGRESS.md)** - Testing progress
- **[Test Coverage Checklist](./test-coverage-checklist.md)** - Testing checklist

## 📜 Historical

- **[Old README](./OLD_README.md)** - Previous README before restructuring

---

## Navigation

- **[← Back to Main Docs](../README.md)**
- **[← Back to Root README](../../README.md)**
- **[User Documentation →](../user/)**
- **[Ideas & Proposals →](../ideas/)**
