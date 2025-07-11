use std::process::Command;

fn main() {
    // Get version from git tag or Cargo.toml
    let version = get_version();

    // Set the version as an environment variable for compile-time access
    println!("cargo:rustc-env=VEXY_JSON_VERSION={}", version);

    // Rerun if git HEAD changes
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs/tags");
}

fn get_version() -> String {
    // First try to get version from git tag
    if let Ok(output) = Command::new("git")
        .args(&["describe", "--exact-match", "--tags"])
        .output()
    {
        if output.status.success() {
            let tag = String::from_utf8_lossy(&output.stdout).trim().to_string();
            // Remove 'v' prefix if present
            return tag.strip_prefix('v').unwrap_or(&tag).to_string();
        }
    }

    // Try to get the most recent tag with commit info
    if let Ok(output) = Command::new("git")
        .args(&["describe", "--tags", "--always"])
        .output()
    {
        if output.status.success() {
            let tag = String::from_utf8_lossy(&output.stdout).trim().to_string();
            // Check if this looks like a version tag
            if tag.starts_with('v') || tag.chars().next().map_or(false, |c| c.is_numeric()) {
                let version = tag.strip_prefix('v').unwrap_or(&tag);
                // If we have commits since the tag, append -dev
                if version.contains('-') {
                    let parts: Vec<&str> = version.split('-').collect();
                    if parts.len() > 1 {
                        return format!("{}-dev", parts[0]);
                    }
                }
                return version.to_string();
            }
        }
    }

    // Fallback to Cargo.toml version
    env!("CARGO_PKG_VERSION").to_string()
}
