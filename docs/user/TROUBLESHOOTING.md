# Troubleshooting Guide

Common issues and solutions for LazyMVN.

## 🔍 Quick Diagnostics

### Check Logs

```bash
# View debug logs
tail -f ~/.local/share/lazymvn/logs/debug.log

# View latest session log
tail -f ~/.local/share/lazymvn/logs/sessions/$(ls -t ~/.local/share/lazymvn/logs/sessions/ | head -1)

# Search for errors
grep -i "error\|exception\|fail" ~/.local/share/lazymvn/logs/debug.log
```

### Verify Configuration

```bash
# Check config file exists
cat ~/.config/lazymvn/lazymvn.toml

# Validate TOML syntax
# (LazyMVN will show parse errors on startup)
lazymvn
```

### Check Cache

```bash
# List cache files
ls -lh ~/.local/share/lazymvn/cache/

# Clear cache
rm -rf ~/.local/share/lazymvn/cache/*

# Restart LazyMVN to rebuild cache
lazymvn
```

## 🐛 Common Issues

### Issue: "POM file not found"

**Symptoms**: LazyMVN can't find `pom.xml`

**Solutions**:
1. Run LazyMVN from project root directory:
   ```bash
   cd /path/to/maven/project
   lazymvn
   ```

2. Verify pom.xml exists:
   ```bash
   ls -l pom.xml
   ```

3. Check for multi-module project:
   - Run from parent module
   - Or use `-pl` flag to specify module

---

### Issue: Maven profiles not appearing

**Symptoms**: Expected profiles don't show in menu

**Solutions**:
1. Clear profile cache:
   ```bash
   rm ~/.local/share/lazymvn/cache/profiles.json
   ```

2. Verify profiles in POM:
   ```bash
   mvn help:all-profiles
   ```

3. Check debug logs:
   ```bash
   grep "profile" ~/.local/share/lazymvn/logs/debug.log
   ```

---

### Issue: Custom goals not working

**Symptoms**: Custom goals from config don't appear

**Solutions**:
1. Verify config syntax:
   ```toml
   [[custom_goals]]
   name = "My Goal"
   command = "clean install"
   description = "Custom goal"
   ```

2. Restart LazyMVN to reload config

3. Check for TOML parse errors:
   ```bash
   grep -i "parse\|toml" ~/.local/share/lazymvn/logs/debug.log
   ```

---

### Issue: Log colorization not working

**Symptoms**: Logs appear without colors

**Solutions**:
1. Check terminal supports ANSI colors:
   ```bash
   echo $TERM
   # Should show xterm-256color or similar
   ```

2. Enable color in config:
   ```toml
   [display]
   colorize = true
   ```

3. Try different terminal emulator (e.g., iTerm2, Windows Terminal)

---

### Issue: Maven process won't stop

**Symptoms**: Maven continues running after closing LazyMVN

**Solutions**:
1. Check process cleanup logs:
   ```bash
   grep -i "cleanup\|kill" ~/.local/share/lazymvn/logs/debug.log
   ```

2. Manually kill process:
   ```bash
   # Find PID
   ps aux | grep mvn
   
   # Kill process
   kill -9 <PID>
   ```

3. Verify process cleanup is enabled:
   ```toml
   [maven]
   cleanup_on_exit = true
   ```

---

### Issue: Configuration changes not taking effect

**Symptoms**: Changes to `lazymvn.toml` don't apply

**Solutions**:
1. Verify live reload is enabled:
   ```toml
   [config]
   live_reload = true
   ```

2. Check file permissions:
   ```bash
   ls -l ~/.config/lazymvn/lazymvn.toml
   ```

3. Manually restart LazyMVN

4. Check for config errors:
   ```bash
   grep -i "config\|reload" ~/.local/share/lazymvn/logs/debug.log
   ```

---

### Issue: Cache is stale

**Symptoms**: Old profiles/goals still showing after POM changes

**Solutions**:
1. Force cache refresh with `Ctrl+K`

2. Clear all caches:
   ```bash
   rm -rf ~/.local/share/lazymvn/cache/*
   ```

3. Enable auto-invalidation (coming soon):
   ```toml
   [cache]
   auto_invalidate = true
   ```

---

### Issue: Logs growing too large

**Symptoms**: Debug log consumes disk space

**Solutions**:
1. Enable log rotation:
   ```toml
   [logging]
   rotation = true
   max_size = "10MB"
   max_backups = 5
   ```

2. Manually clean old logs:
   ```bash
   # Remove logs older than 7 days
   find ~/.local/share/lazymvn/logs/sessions/ -type f -mtime +7 -delete
   ```

3. Reduce log level:
   ```toml
   [logging]
   level = "info"  # instead of "debug"
   ```

---

### Issue: Java agent not loading

**Symptoms**: Debug features don't work

**Solutions**:
1. Verify agent JAR exists:
   ```bash
   ls -l ~/.local/share/lazymvn/agent/lazymvn-agent.jar
   ```

2. Rebuild agent:
   ```bash
   cargo build --release
   ```

3. Check Maven supports agents:
   ```bash
   mvn -version
   # Verify Java version is compatible
   ```

---

### Issue: Spring Boot properties not overriding

**Symptoms**: Custom properties don't apply

**Solutions**:
1. Verify property syntax:
   ```toml
   [spring_boot]
   properties = { "server.port" = "8081" }
   ```

2. Check property precedence:
   - LazyMVN properties should override application.properties
   - But not command-line `-D` flags

3. View effective properties:
   ```bash
   grep -i "spring\|property" ~/.local/share/lazymvn/logs/debug.log
   ```

---

## 🔧 Advanced Diagnostics

### Enable Debug Logging

```bash
# Set environment variable
export RUST_LOG=debug

# Run LazyMVN
lazymvn

# Check all debug output
tail -f ~/.local/share/lazymvn/logs/debug.log
```

### Check System Resources

```bash
# Check disk space
df -h ~/.local/share/lazymvn

# Check memory usage
ps aux | grep lazymvn

# Monitor file handles
lsof | grep lazymvn
```

### Verify Dependencies

```bash
# Check Maven installation
mvn -version

# Verify Java version
java -version

# Check terminal capabilities
echo $TERM
tput colors
```

### Test Configuration

```bash
# Validate TOML syntax with a parser
cat ~/.config/lazymvn/lazymvn.toml | toml-cli validate

# Or use LazyMVN's built-in validation
lazymvn --validate-config
```

## 🆘 Getting Help

### Before Reporting Issues

1. **Check logs**: Review debug logs for error messages
2. **Clear cache**: Remove cache and try again
3. **Update**: Ensure you're running the latest version
4. **Minimal config**: Test with minimal configuration

### Reporting Bugs

Include the following information:

1. **LazyMVN version**: `lazymvn --version`
2. **Operating system**: `uname -a`
3. **Maven version**: `mvn -version`
4. **Configuration**: Your `lazymvn.toml` (remove sensitive data)
5. **Logs**: Relevant excerpts from debug log
6. **Steps to reproduce**: Clear steps to trigger the issue

### Where to Report

- **GitHub Issues**: https://github.com/your-repo/lazymvn/issues
- **Discussions**: https://github.com/your-repo/lazymvn/discussions

## 📚 Related Documentation

- **Configuration**: `docs/user/README.md`
- **Logging**: `docs/user/LOGGING_CONFIG.md`
- **Caching**: `docs/user/CACHING.md`
- **Architecture**: `docs/user/ARCHITECTURE.md`

## 🔄 Reset Everything

If all else fails, start fresh:

```bash
# Backup current config (optional)
cp ~/.config/lazymvn/lazymvn.toml ~/.config/lazymvn/lazymvn.toml.backup

# Remove all LazyMVN data
rm -rf ~/.config/lazymvn
rm -rf ~/.local/share/lazymvn

# Reinstall LazyMVN
cargo install --path . --force

# Create new config
lazymvn --init

# Run LazyMVN
lazymvn
```
