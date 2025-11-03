# Fix Spring Boot Plugin 1.4.13 Not Found

## Problem

```
[ERROR] Plugin org.springframework.boot:spring-boot-maven-plugin:jar:1.4.13 was not found
This failure was cached in the local repository
```

## Solutions (In Order of Preference)

### Solution 1: Purge Entire Spring Boot Cache (Recommended)

**In LazyMVN:**
1. Navigate to **root module** (parent POM)
2. Press `p` to purge
3. Press `b` to rebuild

**OR Manually:**
```powershell
# Remove entire Spring Boot plugin cache
Remove-Item -Recurse -Force "$env:USERPROFILE\.m2\repository\org\springframework\boot"

# Rebuild
mvn clean install -U
```

### Solution 2: Force Update with -U Flag

LazyMVN doesn't have `-U` flag yet. Use terminal:

```powershell
cd C:\ProgramData\espdev\workspace\foo\foo-api
mvn clean install -U
```

The `-U` flag forces Maven to:
- Check for updates even if cached as "not found"
- Re-attempt failed downloads
- Ignore cache timeout

### Solution 3: Check Artifactory Availability

**Test plugin availability:**
```powershell
# Try to download manually
curl -I https://foo.pop.sf.foo.foo.fr/artifactory/entreprise-maven/org/springframework/boot/spring-boot-maven-plugin/1.4.13/spring-boot-maven-plugin-1.4.13.pom
```

**Expected results:**
- ✅ `200 OK` → Plugin exists, cache issue
- ❌ `404 Not Found` → Plugin missing, need alternative

### Solution 4: Update Plugin Version (If Plugin Missing)

If plugin truly doesn't exist in Artifactory, update POM:

**Option A: Spring Boot 1.5.x (compatible)**
```xml
<plugin>
    <groupId>org.springframework.boot</groupId>
    <artifactId>spring-boot-maven-plugin</artifactId>
    <version>1.5.22.RELEASE</version> <!-- Last 1.x version -->
</plugin>
```

**Option B: Check parent POM**
```xml
<parent>
    <groupId>org.springframework.boot</groupId>
    <artifactId>spring-boot-starter-parent</artifactId>
    <version>1.4.13.RELEASE</version> <!-- or 1.5.22.RELEASE -->
</parent>
```

### Solution 5: Use Central Repository as Fallback

Add to `maven_settings.xml`:

```xml
<mirrors>
    <mirror>
        <id>central-fallback</id>
        <mirrorOf>!entreprise-maven</mirrorOf>
        <url>https://repo.maven.apache.org/maven2</url>
    </mirror>
</mirrors>
```

This allows Maven Central as backup if Artifactory fails.

## Which Module Has the Issue?

**Find it:**
1. In LazyMVN, press `2` to focus Modules
2. Look for modules with Spring Boot in their POM
3. Try purging each one individually

**OR check POMs:**
```powershell
# Find all modules using Spring Boot plugin 1.4.13
Get-ChildItem -Recurse -Filter pom.xml | Select-String "spring-boot-maven-plugin" -Context 2,2
```

## Why Purge Didn't Work Completely?

**Possible reasons:**

1. **Wrong scope**: Purged a child module, but parent declares the plugin
2. **Multiple modules**: Each module needs individual purge
3. **Network cache**: Artifactory itself caching the 404
4. **Settings.xml**: Repository settings preventing proper resolution

## Recommended Workflow

```powershell
# 1. Nuclear option - clear entire local repo for this project
cd C:\ProgramData\espdev\workspace\foo\foo-api
mvn dependency:purge-local-repository -DreResolve=false

# 2. Force update from remote
mvn clean install -U

# 3. If still fails, check Artifactory
curl -I https://foo.pop.sf.foo.foo.fr/artifactory/entreprise-maven/org/springframework/boot/spring-boot-maven-plugin/1.4.13/spring-boot-maven-plugin-1.4.13.jar
```

## Future LazyMVN Enhancement

**Feature request:** Add `-U` (force update) flag support
- Key: `Shift+U` or in Flags menu
- Command: Adds `-U` to any Maven command
- Use case: Exactly this scenario

## See Also

- [PURGE_LOCAL_REPOSITORY.md](PURGE_LOCAL_REPOSITORY.md)
- [Maven Settings Management](MAVEN_SETTINGS.md)
