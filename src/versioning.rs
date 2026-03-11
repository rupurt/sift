pub fn compose_version_string(
    package_version: &str,
    git_sha: Option<&str>,
    release_build: bool,
) -> String {
    let short_sha = git_sha
        .and_then(normalize_sha)
        .unwrap_or_else(|| "unknown".to_string());

    if release_build {
        format!("{package_version} ({short_sha})")
    } else {
        format!("{package_version}-dev ({short_sha})")
    }
}

pub fn normalize_sha(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.chars().take(7).collect())
    }
}

pub fn is_release_build(profile: &str, release_override: Option<&str>) -> bool {
    release_override.map(is_truthy).unwrap_or(false) || matches!(profile, "release" | "dist")
}

fn is_truthy(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}

#[cfg(test)]
mod tests {
    use super::{compose_version_string, is_release_build, normalize_sha};

    #[test]
    fn renders_dev_version_with_short_sha() {
        assert_eq!(
            compose_version_string("0.1.0", Some("abcdef123456"), false),
            "0.1.0-dev (abcdef1)"
        );
    }

    #[test]
    fn renders_release_version_without_dev_suffix() {
        assert_eq!(
            compose_version_string("0.1.0", Some("abcdef123456"), true),
            "0.1.0 (abcdef1)"
        );
    }

    #[test]
    fn falls_back_to_unknown_when_sha_is_missing() {
        assert_eq!(
            compose_version_string("0.1.0", None, false),
            "0.1.0-dev (unknown)"
        );
        assert_eq!(
            compose_version_string("0.1.0", Some("   "), true),
            "0.1.0 (unknown)"
        );
    }

    #[test]
    fn detects_release_profiles_and_overrides() {
        assert!(is_release_build("release", None));
        assert!(is_release_build("dist", None));
        assert!(is_release_build("debug", Some("true")));
        assert!(!is_release_build("debug", None));
    }

    #[test]
    fn normalizes_sha_to_seven_characters() {
        assert_eq!(normalize_sha("abcdef123456"), Some("abcdef1".to_string()));
        assert_eq!(normalize_sha("abc"), Some("abc".to_string()));
    }
}
