use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=agent/pom.xml");
    println!("cargo:rerun-if-changed=agent/src/");
    
    // Build the Java agent JAR using Maven
    let status = Command::new("mvn")
        .args(["-f", "agent/pom.xml", "clean", "package", "-q"])
        .status();
    
    match status {
        Ok(exit_status) if exit_status.success() => {
            println!("cargo:warning=Java agent built successfully");
        }
        Ok(exit_status) => {
            println!(
                "cargo:warning=Java agent build failed with exit code: {}",
                exit_status.code().unwrap_or(-1)
            );
        }
        Err(e) => {
            println!("cargo:warning=Failed to run mvn: {}. Agent will not be available.", e);
        }
    }
}
