//! Maven command builder with fluent API

use std::path::{Path, PathBuf};

/// A fluent builder for constructing Maven commands
///
/// # Examples
///
/// ```
/// use maven_command_builder::MavenCommandBuilder;
/// use std::path::Path;
///
/// let cmd = MavenCommandBuilder::new(Path::new("/project"))
///     .goal("clean")
///     .goal("install")
///     .profile("dev")
///     .skip_tests(true)
///     .build();
///
/// assert!(cmd.contains("clean"));
/// assert!(cmd.contains("install"));
/// assert!(cmd.contains("dev"));
/// assert!(cmd.contains("-DskipTests"));
/// ```
#[derive(Debug, Clone)]
pub struct MavenCommandBuilder {
    project_root: PathBuf,
    maven_executable: Option<String>,
    goals: Vec<String>,
    profiles: Vec<String>,
    properties: Vec<(String, String)>,
    flags: Vec<String>,
    module: Option<String>,
    settings_file: Option<String>,
    threads: Option<String>,
    use_file_flag: bool,
    offline: bool,
    update_snapshots: bool,
    skip_tests: bool,
    also_make: bool,
    also_make_dependents: bool,
}

impl MavenCommandBuilder {
    /// Create a new Maven command builder for the given project root
    pub fn new(project_root: &Path) -> Self {
        Self {
            project_root: project_root.to_path_buf(),
            maven_executable: None,
            goals: Vec::new(),
            profiles: Vec::new(),
            properties: Vec::new(),
            flags: Vec::new(),
            module: None,
            settings_file: None,
            threads: None,
            use_file_flag: false,
            offline: false,
            update_snapshots: false,
            skip_tests: false,
            also_make: false,
            also_make_dependents: false,
        }
    }

    /// Set a custom Maven executable (default auto-detects mvnw or mvn)
    pub fn maven_executable(mut self, executable: impl Into<String>) -> Self {
        self.maven_executable = Some(executable.into());
        self
    }

    /// Add a Maven goal (e.g., "clean", "install", "package")
    pub fn goal(mut self, goal: impl Into<String>) -> Self {
        self.goals.push(goal.into());
        self
    }

    /// Add multiple Maven goals
    pub fn goals<I, S>(mut self, goals: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.goals.extend(goals.into_iter().map(|g| g.into()));
        self
    }

    /// Add a profile to activate
    pub fn profile(mut self, profile: impl Into<String>) -> Self {
        self.profiles.push(profile.into());
        self
    }

    /// Add multiple profiles to activate
    pub fn profiles<I, S>(mut self, profiles: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.profiles.extend(profiles.into_iter().map(|p| p.into()));
        self
    }

    /// Add a Maven property (-Dkey=value)
    pub fn property(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.properties.push((key.into(), value.into()));
        self
    }

    /// Add a custom flag (e.g., "--debug", "-X")
    pub fn flag(mut self, flag: impl Into<String>) -> Self {
        self.flags.push(flag.into());
        self
    }

    /// Specify a module to build (-pl or -f)
    pub fn module(mut self, module: impl Into<String>) -> Self {
        self.module = Some(module.into());
        self
    }

    /// Specify a settings file (--settings)
    pub fn settings_file(mut self, path: impl Into<String>) -> Self {
        self.settings_file = Some(path.into());
        self
    }

    /// Set number of threads (-T)
    pub fn threads(mut self, threads: impl Into<String>) -> Self {
        self.threads = Some(threads.into());
        self
    }

    /// Use -f (file flag) instead of -pl for module selection
    pub fn use_file_flag(mut self, use_it: bool) -> Self {
        self.use_file_flag = use_it;
        self
    }

    /// Enable offline mode (-o, --offline)
    pub fn offline(mut self, offline: bool) -> Self {
        self.offline = offline;
        self
    }

    /// Update snapshots (-U, --update-snapshots)
    pub fn update_snapshots(mut self, update: bool) -> Self {
        self.update_snapshots = update;
        self
    }

    /// Skip tests (-DskipTests)
    pub fn skip_tests(mut self, skip: bool) -> Self {
        self.skip_tests = skip;
        self
    }

    /// Add --also-make flag
    pub fn also_make(mut self, enable: bool) -> Self {
        self.also_make = enable;
        self
    }

    /// Add --also-make-dependents flag
    pub fn also_make_dependents(mut self, enable: bool) -> Self {
        self.also_make_dependents = enable;
        self
    }

    /// Build the command as a vector of arguments (for Process::Command)
    pub fn build_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        add_settings_file(&mut args, &self.settings_file);
        add_profiles(&mut args, &self.profiles);
        add_module(&mut args, &self.module, self.use_file_flag, &self.project_root);
        add_threads(&mut args, &self.threads);
        add_boolean_flags(&mut args, self);
        add_custom_flags(&mut args, &self.flags);
        add_properties(&mut args, &self.properties);
        add_skip_tests_property(&mut args, self.skip_tests);
        add_goals(&mut args, &self.goals);

        args
    }

    /// Build the command as a shell-friendly string
    ///
    /// # Examples
    ///
    /// ```
    /// use maven_command_builder::MavenCommandBuilder;
    /// use std::path::Path;
    ///
    /// let cmd = MavenCommandBuilder::new(Path::new("/project"))
    ///     .goal("clean")
    ///     .goal("install")
    ///     .build();
    ///
    /// assert!(cmd.contains("mvn") || cmd.contains("mvnw"));
    /// assert!(cmd.contains("clean install"));
    /// ```
    pub fn build(&self) -> String {
        let maven_cmd = self.get_maven_executable();
        let args = self.build_args();
        
        let mut parts = vec![maven_cmd];
        parts.extend(args);
        
        parts.join(" ")
    }

    /// Get the Maven executable to use (auto-detects mvnw or mvn)
    pub fn get_maven_executable(&self) -> String {
        if let Some(exec) = &self.maven_executable {
            return exec.clone();
        }

        // Auto-detect Maven wrapper or system Maven
        #[cfg(unix)]
        {
            if self.project_root.join("mvnw").exists() {
                return "./mvnw".to_string();
            }
        }

        #[cfg(windows)]
        {
            if self.project_root.join("mvnw.bat").exists() {
                return "mvnw.bat".to_string();
            }
            if self.project_root.join("mvnw.cmd").exists() {
                return "mvnw.cmd".to_string();
            }
            if self.project_root.join("mvnw").exists() {
                return "mvnw".to_string();
            }
        }

        // Fallback to system Maven
        #[cfg(windows)]
        {
            "mvn.cmd".to_string()
        }
        #[cfg(not(windows))]
        {
            "mvn".to_string()
        }
    }

    /// Get the project root directory
    pub fn project_root(&self) -> &Path {
        &self.project_root
    }
}

/// Add settings file argument
fn add_settings_file(args: &mut Vec<String>, settings_file: &Option<String>) {
    if let Some(settings) = settings_file {
        args.push("--settings".to_string());
        args.push(settings.clone());
    }
}

/// Add profiles argument
fn add_profiles(args: &mut Vec<String>, profiles: &[String]) {
    if !profiles.is_empty() {
        args.push("-P".to_string());
        args.push(profiles.join(","));
    }
}

/// Add module argument
fn add_module(args: &mut Vec<String>, module: &Option<String>, use_file_flag: bool, project_root: &Path) {
    if let Some(module) = module {
        if module != "." {
            if use_file_flag {
                let module_pom = project_root.join(module).join("pom.xml");
                args.push("-f".to_string());
                args.push(module_pom.to_string_lossy().to_string());
            } else {
                args.push("-pl".to_string());
                args.push(module.clone());
            }
        }
    }
}

/// Add threads argument
fn add_threads(args: &mut Vec<String>, threads: &Option<String>) {
    if let Some(threads) = threads {
        args.push("-T".to_string());
        args.push(threads.clone());
    }
}

/// Add boolean flags
fn add_boolean_flags(args: &mut Vec<String>, builder: &MavenCommandBuilder) {
    if builder.offline {
        args.push("--offline".to_string());
    }
    if builder.update_snapshots {
        args.push("--update-snapshots".to_string());
    }
    if builder.also_make {
        args.push("--also-make".to_string());
    }
    if builder.also_make_dependents {
        args.push("--also-make-dependents".to_string());
    }
}

/// Add custom flags
fn add_custom_flags(args: &mut Vec<String>, flags: &[String]) {
    for flag in flags {
        if let Some(first_flag) = flag.split(',').next() {
            for part in first_flag.split_whitespace() {
                if !part.is_empty() {
                    args.push(part.to_string());
                }
            }
        }
    }
}

/// Add properties
fn add_properties(args: &mut Vec<String>, properties: &[(String, String)]) {
    for (key, value) in properties {
        args.push(format!("-D{}={}", key, value));
    }
}

/// Add skip tests property
fn add_skip_tests_property(args: &mut Vec<String>, skip_tests: bool) {
    if skip_tests {
        args.push("-DskipTests".to_string());
    }
}

/// Add goals
fn add_goals(args: &mut Vec<String>, goals: &[String]) {
    args.extend(goals.iter().cloned());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_command() {
        let cmd = MavenCommandBuilder::new(Path::new("/project"))
            .goal("clean")
            .goal("install")
            .build();

        assert!(cmd.contains("clean"));
        assert!(cmd.contains("install"));
    }

    #[test]
    fn test_with_profiles() {
        let cmd = MavenCommandBuilder::new(Path::new("/project"))
            .goal("package")
            .profile("dev")
            .profile("fast")
            .build();

        assert!(cmd.contains("package"));
        assert!(cmd.contains("-P") && (cmd.contains("dev,fast") || cmd.contains("dev") && cmd.contains("fast")));
    }

    #[test]
    fn test_with_properties() {
        let cmd = MavenCommandBuilder::new(Path::new("/project"))
            .goal("test")
            .property("maven.test.skip", "true")
            .property("log.level", "DEBUG")
            .build();

        assert!(cmd.contains("-Dmaven.test.skip=true"));
        assert!(cmd.contains("-Dlog.level=DEBUG"));
    }

    #[test]
    fn test_skip_tests() {
        let cmd = MavenCommandBuilder::new(Path::new("/project"))
            .goal("package")
            .skip_tests(true)
            .build();

        assert!(cmd.contains("-DskipTests"));
    }

    #[test]
    fn test_with_module() {
        let cmd = MavenCommandBuilder::new(Path::new("/project"))
            .goal("compile")
            .module("my-module")
            .build();

        assert!(cmd.contains("-pl"));
        assert!(cmd.contains("my-module"));
    }

    #[test]
    fn test_with_file_flag() {
        let cmd = MavenCommandBuilder::new(Path::new("/project"))
            .goal("compile")
            .module("my-module")
            .use_file_flag(true)
            .build();

        assert!(cmd.contains("-f"));
        assert!(cmd.contains("my-module"));
        assert!(cmd.contains("pom.xml"));
    }

    #[test]
    fn test_with_threads() {
        let cmd = MavenCommandBuilder::new(Path::new("/project"))
            .goal("package")
            .threads("4")
            .build();

        assert!(cmd.contains("-T"));
        assert!(cmd.contains("4"));
    }

    #[test]
    fn test_offline_mode() {
        let cmd = MavenCommandBuilder::new(Path::new("/project"))
            .goal("verify")
            .offline(true)
            .build();

        assert!(cmd.contains("--offline"));
    }

    #[test]
    fn test_update_snapshots() {
        let cmd = MavenCommandBuilder::new(Path::new("/project"))
            .goal("clean")
            .goal("install")
            .update_snapshots(true)
            .build();

        assert!(cmd.contains("--update-snapshots"));
    }

    #[test]
    fn test_also_make() {
        let cmd = MavenCommandBuilder::new(Path::new("/project"))
            .goal("install")
            .module("backend")
            .also_make(true)
            .build();

        assert!(cmd.contains("--also-make"));
    }

    #[test]
    fn test_complex_command() {
        let cmd = MavenCommandBuilder::new(Path::new("/project"))
            .goal("clean")
            .goal("package")
            .profile("production")
            .property("env", "prod")
            .skip_tests(true)
            .threads("2C")
            .offline(true)
            .build();

        assert!(cmd.contains("clean"));
        assert!(cmd.contains("package"));
        assert!(cmd.contains("-P") && cmd.contains("production"));
        assert!(cmd.contains("-Denv=prod"));
        assert!(cmd.contains("-DskipTests"));
        assert!(cmd.contains("-T"));
        assert!(cmd.contains("2C"));
        assert!(cmd.contains("--offline"));
    }

    #[test]
    fn test_build_args() {
        let builder = MavenCommandBuilder::new(Path::new("/project"))
            .goal("test")
            .profile("dev")
            .skip_tests(true);

        let args = builder.build_args();

        assert!(args.iter().any(|a| a.contains("dev")));  // Profile in -Pdev format
        assert!(args.contains(&"-DskipTests".to_string()));
        assert!(args.contains(&"test".to_string()));
    }
}
