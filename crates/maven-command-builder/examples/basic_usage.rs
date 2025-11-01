//! Basic usage examples for maven-command-builder

use maven_command_builder::MavenCommandBuilder;
use std::path::Path;

fn main() {
    println!("=== Maven Command Builder Examples ===\n");

    // Example 1: Simple build
    println!("1. Simple clean and install:");
    let cmd = MavenCommandBuilder::new(Path::new("."))
        .goal("clean")
        .goal("install")
        .build();
    println!("   {}\n", cmd);

    // Example 2: With profiles
    println!("2. Build with profiles:");
    let cmd = MavenCommandBuilder::new(Path::new("."))
        .goal("package")
        .profile("production")
        .profile("optimized")
        .build();
    println!("   {}\n", cmd);

    // Example 3: With properties
    println!("3. Build with properties:");
    let cmd = MavenCommandBuilder::new(Path::new("."))
        .goal("test")
        .property("test.groups", "integration")
        .property("log.level", "DEBUG")
        .build();
    println!("   {}\n", cmd);

    // Example 4: Skip tests
    println!("4. Fast build (skip tests):");
    let cmd = MavenCommandBuilder::new(Path::new("."))
        .goal("clean")
        .goal("package")
        .skip_tests(true)
        .build();
    println!("   {}\n", cmd);

    // Example 5: Multi-threaded build
    println!("5. Parallel build:");
    let cmd = MavenCommandBuilder::new(Path::new("."))
        .goal("install")
        .threads("2C")
        .offline(true)
        .build();
    println!("   {}\n", cmd);

    // Example 6: Module-specific build
    println!("6. Build specific module:");
    let cmd = MavenCommandBuilder::new(Path::new("."))
        .goal("install")
        .module("backend-api")
        .also_make(true)
        .build();
    println!("   {}\n", cmd);

    // Example 7: Spring Boot run
    println!("7. Spring Boot development:");
    let cmd = MavenCommandBuilder::new(Path::new("."))
        .goal("spring-boot:run")
        .profile("dev")
        .property("spring.profiles.active", "development")
        .build();
    println!("   {}\n", cmd);

    // Example 8: Complex build
    println!("8. Complex CI/CD build:");
    let cmd = MavenCommandBuilder::new(Path::new("."))
        .goal("clean")
        .goal("deploy")
        .profile("ci")
        .profile("release")
        .property("maven.javadoc.skip", "false")
        .property("gpg.skip", "false")
        .threads("2C")
        .update_snapshots(true)
        .build();
    println!("   {}\n", cmd);

    // Example 9: Get just the args
    println!("9. Build args only (for Process::Command):");
    let builder = MavenCommandBuilder::new(Path::new("."))
        .goal("verify")
        .skip_tests(true);
    let args = builder.build_args();
    println!("   Args: {:?}\n", args);

    // Example 10: Custom Maven executable
    println!("10. Custom Maven executable:");
    let cmd = MavenCommandBuilder::new(Path::new("."))
        .maven_executable("/usr/local/bin/mvn")
        .goal("clean")
        .goal("compile")
        .build();
    println!("    {}\n", cmd);
}
