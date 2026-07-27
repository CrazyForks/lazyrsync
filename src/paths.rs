use chrono::format::{Item, StrftimeItems};

pub fn expand_tilde(s: &str) -> String {
    let home = std::env::var("HOME").ok().filter(|h| !h.is_empty());
    match (s, home) {
        ("~", Some(home)) => home,
        (p, Some(home)) if p.starts_with("~/") => format!("{home}/{}", &p[2..]),
        _ => s.to_string(),
    }
}

fn hostname() -> Option<String> {
    for path in ["/etc/hostname", "/proc/sys/kernel/hostname"] {
        if let Ok(text) = std::fs::read_to_string(path) {
            let name = text.trim();
            if !name.is_empty() {
                return Some(name.to_string());
            }
        }
    }
    ["HOSTNAME", "HOST"]
        .iter()
        .filter_map(|k| std::env::var(k).ok())
        .find(|v| !v.is_empty())
}

fn format_now(fmt: &str, utc: bool) -> Option<String> {
    if StrftimeItems::new(fmt).any(|item| matches!(item, Item::Error)) {
        return None;
    }
    Some(if utc {
        chrono::Utc::now().format(fmt).to_string()
    } else {
        chrono::Local::now().format(fmt).to_string()
    })
}

fn placeholder(name: &str, arg: Option<&str>) -> Option<String> {
    match name {
        "now" => format_now(arg.unwrap_or("%Y-%m-%d"), false),
        "utcnow" => format_now(arg.unwrap_or("%Y-%m-%d"), true),
        "hostname" if arg.is_none() => hostname(),
        "user" if arg.is_none() => ["USER", "LOGNAME"]
            .iter()
            .filter_map(|k| std::env::var(k).ok())
            .find(|v| !v.is_empty()),
        _ => None,
    }
}

pub fn expand_placeholders(s: &str) -> String {
    if !s.contains('{') && !s.contains('}') {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '{' if chars.peek() == Some(&'{') => {
                chars.next();
                out.push('{');
            }
            '}' if chars.peek() == Some(&'}') => {
                chars.next();
                out.push('}');
            }
            '{' => {
                let mut body = String::new();
                let mut closed = false;
                for n in chars.by_ref() {
                    if n == '}' {
                        closed = true;
                        break;
                    }
                    body.push(n);
                }
                let (name, arg) = match body.split_once(':') {
                    Some((n, a)) => (n, Some(a)),
                    None => (body.as_str(), None),
                };
                match (closed, placeholder(name, arg)) {
                    (true, Some(val)) => out.push_str(&val),
                    (true, None) => {
                        out.push('{');
                        out.push_str(&body);
                        out.push('}');
                    }
                    (false, _) => {
                        out.push('{');
                        out.push_str(&body);
                    }
                }
            }
            c => out.push(c),
        }
    }
    out
}

pub fn expand_env(s: &str) -> String {
    if !s.contains('$') {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '$' {
            out.push(c);
            continue;
        }
        match chars.peek() {
            Some('$') => {
                chars.next();
                out.push('$');
            }
            Some('{') => {
                chars.next();
                let mut name = String::new();
                let mut closed = false;
                for n in chars.by_ref() {
                    if n == '}' {
                        closed = true;
                        break;
                    }
                    name.push(n);
                }
                match (closed, std::env::var(&name)) {
                    (true, Ok(val)) => out.push_str(&val),
                    (true, Err(_)) => {
                        out.push_str("${");
                        out.push_str(&name);
                        out.push('}');
                    }
                    (false, _) => {
                        out.push_str("${");
                        out.push_str(&name);
                    }
                }
            }
            _ => {
                let mut name = String::new();
                while let Some(&n) = chars.peek() {
                    if n.is_ascii_alphanumeric() || n == '_' {
                        name.push(n);
                        chars.next();
                    } else {
                        break;
                    }
                }
                if name.is_empty() {
                    out.push('$');
                } else if let Ok(val) = std::env::var(&name) {
                    out.push_str(&val);
                } else {
                    out.push('$');
                    out.push_str(&name);
                }
            }
        }
    }
    out
}

pub fn expand_path(s: &str) -> String {
    expand_env(&expand_tilde(&expand_placeholders(s)))
}

pub fn path_hits(buffer: &str) -> Vec<String> {
    if buffer.contains('@') || (buffer.contains(':') && !buffer.starts_with('/')) {
        return Vec::new();
    }
    let (dir_disp, frag) = match buffer.rsplit_once('/') {
        Some((d, f)) => (d.to_string(), f.to_string()),
        None => (String::new(), buffer.to_string()),
    };
    let dir_real = if dir_disp.is_empty() {
        if buffer.starts_with('/') {
            "/".to_string()
        } else {
            ".".to_string()
        }
    } else {
        expand_tilde(&dir_disp)
    };
    let smart_sensitive = frag.chars().any(|c| c.is_uppercase());
    let frag_lower = frag.to_lowercase();
    let mut hits: Vec<String> = Vec::new();
    if let Ok(rd) = std::fs::read_dir(&dir_real) {
        for entry in rd.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if !frag.starts_with('.') && name.starts_with('.') {
                continue;
            }
            let matches = if smart_sensitive {
                name.starts_with(&frag)
            } else {
                name.to_lowercase().starts_with(&frag_lower)
            };
            if matches {
                let mut full = if dir_disp.is_empty() {
                    if buffer.starts_with('/') {
                        format!("/{name}")
                    } else {
                        name.clone()
                    }
                } else {
                    format!("{dir_disp}/{name}")
                };
                if entry.path().is_dir() {
                    full.push('/');
                }
                hits.push(full);
            }
        }
    }
    hits.sort();
    hits
}

pub fn complete_path(buffer: &str) -> String {
    let hits = path_hits(buffer);
    match hits.len() {
        0 => buffer.to_string(),
        1 => hits[0].clone(),
        _ => {
            let first = &hits[0];
            let mut len = first.len();
            for h in &hits[1..] {
                len = first
                    .chars()
                    .zip(h.chars())
                    .take_while(|(a, b)| a == b)
                    .count()
                    .min(len);
            }
            first.chars().take(len).collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{complete_path, expand_env, expand_path, expand_placeholders};

    #[test]
    fn now_expands_with_an_explicit_format() {
        let want = chrono::Local::now().format("%Y-%m-%d").to_string();
        assert_eq!(
            expand_placeholders("/bak/{now:%Y-%m-%d}"),
            format!("/bak/{want}")
        );
    }

    #[test]
    fn bare_now_defaults_to_iso_date() {
        let want = chrono::Local::now().format("%Y-%m-%d").to_string();
        assert_eq!(expand_placeholders("/bak/{now}"), format!("/bak/{want}"));
    }

    #[test]
    fn literal_percent_in_a_path_is_never_touched() {
        assert_eq!(expand_placeholders("/mnt/100%done/"), "/mnt/100%done/");
        assert_eq!(expand_path("/mnt/100%done/"), "/mnt/100%done/");
        assert_eq!(expand_placeholders("/mnt/50%off/"), "/mnt/50%off/");
    }

    #[test]
    fn paths_without_braces_are_untouched() {
        assert_eq!(expand_placeholders("/bak/nightly"), "/bak/nightly");
    }

    #[test]
    fn unknown_placeholder_is_left_literal() {
        assert_eq!(expand_placeholders("/bak/{nope}"), "/bak/{nope}");
        assert_eq!(expand_placeholders("/bak/{now:%E}"), "/bak/{now:%E}");
    }

    #[test]
    fn unclosed_placeholder_is_left_literal() {
        assert_eq!(expand_placeholders("/bak/{now"), "/bak/{now");
    }

    #[test]
    fn doubled_braces_are_literal_braces() {
        assert_eq!(expand_placeholders("/bak/{{now}}"), "/bak/{now}");
    }

    #[test]
    fn user_placeholder_matches_the_environment() {
        let want = std::env::var("USER").unwrap();
        assert_eq!(expand_placeholders("/bak/{user}"), format!("/bak/{want}"));
    }

    #[test]
    fn hostname_placeholder_resolves_or_stays_literal() {
        let got = expand_placeholders("/bak/{hostname}");
        assert!(got == "/bak/{hostname}" || (got.starts_with("/bak/") && got.len() > 5));
    }

    #[test]
    fn env_vars_expand_in_both_forms() {
        std::env::set_var("LR_TEST_ROOT", "/mnt/disk");
        assert_eq!(expand_env("$LR_TEST_ROOT/x"), "/mnt/disk/x");
        assert_eq!(expand_env("${LR_TEST_ROOT}x"), "/mnt/diskx");
    }

    #[test]
    fn undefined_env_var_is_left_literal() {
        assert_eq!(
            expand_env("/bak/$LR_TEST_MISSING/x"),
            "/bak/$LR_TEST_MISSING/x"
        );
        assert_eq!(
            expand_env("/bak/${LR_TEST_MISSING}"),
            "/bak/${LR_TEST_MISSING}"
        );
    }

    #[test]
    fn double_dollar_is_a_literal_dollar() {
        assert_eq!(expand_env("/bak/$$HOME"), "/bak/$HOME");
    }

    #[test]
    fn env_values_are_not_rescanned_for_placeholders() {
        std::env::set_var("LR_TEST_BRACE", "{now}");
        assert_eq!(expand_path("/bak/$LR_TEST_BRACE"), "/bak/{now}");
    }

    #[test]
    fn tilde_placeholder_and_env_combine() {
        std::env::set_var("LR_TEST_HOST", "thinkpad");
        let home = std::env::var("HOME").unwrap();
        let day = chrono::Local::now().format("%Y-%m-%d").to_string();
        assert_eq!(
            expand_path("~/bak/$LR_TEST_HOST/{now:%Y-%m-%d}"),
            format!("{home}/bak/thinkpad/{day}")
        );
    }

    #[test]
    fn smart_case_lowercase_query_is_insensitive() {
        let base = std::env::temp_dir().join(format!("lr-comp-i-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(base.join("Archive")).unwrap();
        let got = complete_path(&format!("{}/arc", base.display()));
        let _ = std::fs::remove_dir_all(&base);
        assert_eq!(got, format!("{}/Archive/", base.display()));
    }

    #[test]
    fn smart_case_uppercase_query_is_sensitive() {
        let base = std::env::temp_dir().join(format!("lr-comp-s-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(base.join("archive")).unwrap();
        let typed = format!("{}/Arc", base.display());
        let got = complete_path(&typed);
        let _ = std::fs::remove_dir_all(&base);
        assert_eq!(got, typed);
    }

    #[test]
    fn remote_paths_are_left_untouched() {
        assert_eq!(complete_path("me@vps:/backup/da"), "me@vps:/backup/da");
    }
}
