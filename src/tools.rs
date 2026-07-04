use std::process::Command;

#[derive(Debug, Clone)]
pub struct Tool {
    pub version: Option<String>,
}

pub fn exists(name: &str) -> bool {
    which::which(name).is_ok()
}

pub fn probe(name: &'static str) -> Tool {
    Tool {
        version: version_line(name),
    }
}

pub fn fast_linker() -> Option<&'static str> {
    if exists("mold") {
        Some("mold")
    } else if exists("ld.lld") {
        Some("lld")
    } else {
        None
    }
}

pub fn sccache_health() -> ToolHealth {
    if !exists("sccache") {
        return ToolHealth::Missing;
    }

    match Command::new("sccache").arg("rustc").arg("-vV").output() {
        Ok(output) if output.status.success() => ToolHealth::Ok,
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            ToolHealth::Broken(
                first_line(&stderr)
                    .unwrap_or("unknown sccache error")
                    .to_owned(),
            )
        }
        Err(err) => ToolHealth::Broken(err.to_string()),
    }
}

fn version_line(name: &str) -> Option<String> {
    let output = Command::new(name).arg("--version").output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    text.lines().next().map(str::to_owned)
}

fn first_line(text: &str) -> Option<&str> {
    text.lines().find(|line| !line.trim().is_empty())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolHealth {
    Ok,
    Missing,
    Broken(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gets_first_non_empty_line() {
        assert_eq!(first_line("\n\nhello\nworld"), Some("hello"));
        assert_eq!(first_line("\n\n"), None);
    }
}
