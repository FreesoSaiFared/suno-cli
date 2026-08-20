use crate::auth::BrowserAuth;
use crate::errors::CliError;
#[cfg(windows)]
use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit},
};
#[cfg(windows)]
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use rusqlite::{Connection, OpenFlags};
use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone)]
struct CookieRecord {
    domain: String,
    name: String,
    value: String,
}

#[derive(Debug, Clone)]
struct BrowserSpec {
    browser: String,
    profile: Option<String>,
}

const CHROMIUM_BROWSERS: &[&str] = &[
    "brave", "chrome", "chromium", "edge", "opera", "vivaldi", "whale",
];

/// Native Rust implementation of the browser-cookie subset used by yt-dlp's
/// `--cookies-from-browser` flow. No Python process and no Netscape cookie file
/// are involved: only *.suno.com cookies are read and returned in memory.
pub fn extract_browser_auth(spec: Option<&str>) -> Result<BrowserAuth, CliError> {
    if let Some(spec) = spec {
        let parsed = parse_browser_spec(spec)?;
        let cookies = extract_for_spec(&parsed)?;
        return records_to_browser_auth(&parsed.browser, cookies);
    }

    let mut failures = Vec::new();
    for browser in [
        "brave", "chrome", "edge", "chromium", "vivaldi", "opera", "firefox",
    ] {
        let parsed = BrowserSpec {
            browser: browser.to_string(),
            profile: None,
        };
        match extract_for_spec(&parsed)
            .and_then(|cookies| records_to_browser_auth(browser, cookies))
        {
            Ok(auth) => return Ok(auth),
            Err(err) => failures.push(format!("{browser}: {err}")),
        }
    }

    Err(CliError::Config(format!(
        "No usable Suno browser session found. Native browser-cookie extraction tried: {}",
        failures.join(" | ")
    )))
}

fn parse_browser_spec(input: &str) -> Result<BrowserSpec, CliError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(CliError::InvalidInput(
            "empty browser specification; expected BROWSER[:PROFILE]".into(),
        ));
    }

    let (browser, profile) = match trimmed.split_once(':') {
        Some((browser, profile)) => (browser, Some(profile)),
        None => (trimmed, None),
    };
    let browser = browser.trim().to_ascii_lowercase();
    if browser != "firefox" && !CHROMIUM_BROWSERS.contains(&browser.as_str()) {
        return Err(CliError::InvalidInput(format!(
            "unsupported browser '{browser}'; supported: brave, chrome, chromium, edge, opera, vivaldi, whale, firefox"
        )));
    }

    let profile = profile
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);
    Ok(BrowserSpec { browser, profile })
}

fn extract_for_spec(spec: &BrowserSpec) -> Result<Vec<CookieRecord>, CliError> {
    if spec.browser == "firefox" {
        extract_firefox(spec.profile.as_deref())
    } else {
        extract_chromium(&spec.browser, spec.profile.as_deref())
    }
}

fn records_to_browser_auth(
    browser: &str,
    cookies: Vec<CookieRecord>,
) -> Result<BrowserAuth, CliError> {
    let mut seen = HashSet::new();
    let mut header_parts = Vec::new();
    let mut generic_clerk = None;
    let mut auth_domain_clerk = None;
    let mut device_id = None;

    for cookie in cookies {
        if !is_suno_domain(&cookie.domain) {
            continue;
        }
        if cookie.name == "__client" && !cookie.value.is_empty() {
            if cookie.domain.eq_ignore_ascii_case("auth.suno.com")
                || cookie.domain.eq_ignore_ascii_case(".auth.suno.com")
            {
                auth_domain_clerk = Some(cookie.value.clone());
            } else if generic_clerk.is_none() {
                generic_clerk = Some(cookie.value.clone());
            }
        }
        if cookie.name == "ajs_anonymous_id" && device_id.is_none() {
            device_id = sanitize_device_id(&cookie.value);
        }
        if seen.insert((cookie.name.clone(), cookie.domain.clone())) {
            header_parts.push(format!("{}={}", cookie.name, cookie.value));
        }
    }

    let clerk_client_cookie = auth_domain_clerk.or(generic_clerk).ok_or_else(|| {
        CliError::Config(format!(
            "{browser} cookies were readable, but no Suno Clerk __client session cookie was present"
        ))
    })?;

    eprintln!("Found Suno session in {browser} via native browser-cookie extraction");
    Ok(BrowserAuth {
        clerk_client_cookie,
        cookie_header: header_parts.join("; "),
        device_id,
    })
}

fn is_suno_domain(domain: &str) -> bool {
    let domain = domain.trim_start_matches('.').to_ascii_lowercase();
    domain == "suno.com" || domain.ends_with(".suno.com")
}

fn sanitize_device_id(value: &str) -> Option<String> {
    let value = value
        .trim()
        .replace("%22", "\"")
        .trim_matches('"')
        .trim_matches('\'')
        .trim()
        .to_string();
    (!value.is_empty() && !value.contains(';')).then_some(value)
}

fn extract_chromium(browser: &str, profile: Option<&str>) -> Result<Vec<CookieRecord>, CliError> {
    #[cfg(not(windows))]
    {
        let _ = (browser, profile);
        Err(CliError::Config(
            "native Chromium cookie extraction is currently implemented for Windows".into(),
        ))
    }

    #[cfg(windows)]
    {
        let browser_root = chromium_browser_root(browser)?;
        let search_root = resolve_profile_root(browser, &browser_root, profile)?;
        let database = newest_named_file(&search_root, "Cookies", 5).ok_or_else(|| {
            CliError::Config(format!(
                "could not find {browser} Cookies database under {}",
                search_root.display()
            ))
        })?;

        let conn = open_browser_database(&database)?;
        let meta_version = chromium_meta_version(&conn);
        chromium_secure_column(&conn)?;
        let sql = "SELECT host_key, name, value, encrypted_value FROM cookies \
                   WHERE host_key = 'suno.com' OR host_key LIKE '%.suno.com'";
        let mut stmt = conn
            .prepare(sql)
            .map_err(|e| CliError::Config(format!("cannot query Chromium cookies: {e}")))?;
        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Vec<u8>>(3)?,
                ))
            })
            .map_err(|e| CliError::Config(format!("cannot read Chromium cookies: {e}")))?;

        let decryptor = WindowsChromiumDecryptor::new(&browser_root, meta_version)?;
        let mut cookies = Vec::new();
        let mut undecryptable = 0usize;
        for row in rows {
            let (domain, name, clear_value, encrypted_value) = row
                .map_err(|e| CliError::Config(format!("cannot decode Chromium cookie row: {e}")))?;
            let value = if !clear_value.is_empty() {
                Some(clear_value)
            } else if encrypted_value.is_empty() {
                Some(String::new())
            } else {
                decryptor.decrypt(&encrypted_value)?
            };
            if let Some(value) = value {
                cookies.push(CookieRecord {
                    domain,
                    name,
                    value,
                });
            } else {
                undecryptable += 1;
            }
        }
        if undecryptable > 0 {
            eprintln!("Warning: {undecryptable} {browser} Suno cookie(s) could not be decrypted");
        }
        Ok(cookies)
    }
}

#[cfg(windows)]
fn chromium_browser_root(browser: &str) -> Result<PathBuf, CliError> {
    let local = env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .ok_or_else(|| CliError::Config("LOCALAPPDATA is not set".into()))?;
    let roaming = env::var_os("APPDATA")
        .map(PathBuf::from)
        .ok_or_else(|| CliError::Config("APPDATA is not set".into()))?;

    let path = match browser {
        "brave" => local.join(r"BraveSoftware\Brave-Browser\User Data"),
        "chrome" => local.join(r"Google\Chrome\User Data"),
        "chromium" => local.join(r"Chromium\User Data"),
        "edge" => local.join(r"Microsoft\Edge\User Data"),
        "opera" => roaming.join(r"Opera Software\Opera Stable"),
        "vivaldi" => local.join(r"Vivaldi\User Data"),
        "whale" => local.join(r"Naver\Naver Whale\User Data"),
        _ => {
            return Err(CliError::InvalidInput(format!(
                "unsupported Chromium browser '{browser}'"
            )));
        }
    };
    Ok(path)
}

#[cfg(windows)]
fn resolve_profile_root(
    browser: &str,
    browser_root: &Path,
    profile: Option<&str>,
) -> Result<PathBuf, CliError> {
    if browser == "opera" {
        if profile.is_some() {
            return Err(CliError::InvalidInput(
                "opera does not use Chromium-style profile subdirectories".into(),
            ));
        }
        return Ok(browser_root.to_path_buf());
    }

    let Some(profile) = profile else {
        return Ok(browser_root.to_path_buf());
    };
    let profile_path = Path::new(profile);
    if profile_path.is_absolute() || profile.contains('\\') || profile.contains('/') {
        Ok(profile_path.to_path_buf())
    } else {
        Ok(browser_root.join(profile))
    }
}

fn newest_named_file(root: &Path, filename: &str, max_depth: usize) -> Option<PathBuf> {
    fn visit(
        root: &Path,
        filename: &str,
        depth: usize,
        max_depth: usize,
        best: &mut Option<(SystemTime, PathBuf)>,
    ) {
        if depth > max_depth {
            return;
        }
        let Ok(entries) = fs::read_dir(root) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if file_type.is_dir() {
                visit(&path, filename, depth + 1, max_depth, best);
            } else if file_type.is_file()
                && entry
                    .file_name()
                    .to_string_lossy()
                    .eq_ignore_ascii_case(filename)
            {
                let modified = entry
                    .metadata()
                    .and_then(|metadata| metadata.modified())
                    .unwrap_or(SystemTime::UNIX_EPOCH);
                if best.as_ref().is_none_or(|(when, _)| modified > *when) {
                    *best = Some((modified, path));
                }
            }
        }
    }

    let mut best = None;
    visit(root, filename, 0, max_depth, &mut best);
    best.map(|(_, path)| path)
}

fn open_browser_database(path: &Path) -> Result<Connection, CliError> {
    let flags = OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX;

    // Prefer a direct read-only SQLite connection. Unlike yt-dlp's unconditional
    // file-copy approach, this can succeed while Chromium is live when the
    // browser's Windows sharing mode permits readers.
    if let Ok(conn) = Connection::open_with_flags(path, flags) {
        let _ = conn.busy_timeout(std::time::Duration::from_millis(750));
        return Ok(conn);
    }

    // Preserve yt-dlp's copy-before-query fallback for browsers that permit
    // copying but not direct SQLite access.
    let temp = env::temp_dir().join(format!(
        "suno-browser-cookies-{}.sqlite",
        uuid::Uuid::new_v4()
    ));
    fs::copy(path, &temp).map_err(|e| {
        CliError::Config(format!(
            "could not open or copy browser cookie database '{}': {e}. If the browser is locking it, close that browser briefly and retry",
            path.display()
        ))
    })?;
    let conn = Connection::open_with_flags(&temp, flags).map_err(|e| {
        CliError::Config(format!("cannot open copied browser cookie database: {e}"))
    })?;
    let _ = fs::remove_file(&temp);
    Ok(conn)
}

#[cfg(windows)]
fn chromium_meta_version(conn: &Connection) -> i64 {
    conn.query_row("SELECT value FROM meta WHERE key = 'version'", [], |row| {
        row.get::<_, String>(0)
    })
    .ok()
    .and_then(|value| value.parse::<i64>().ok())
    .unwrap_or(0)
}

#[cfg(windows)]
fn chromium_secure_column(conn: &Connection) -> Result<&'static str, CliError> {
    let mut stmt = conn
        .prepare("PRAGMA table_info(cookies)")
        .map_err(|e| CliError::Config(format!("cannot inspect Chromium cookie schema: {e}")))?;
    let names = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|e| CliError::Config(format!("cannot inspect Chromium cookie schema: {e}")))?
        .filter_map(Result::ok)
        .collect::<Vec<_>>();
    if names.iter().any(|name| name == "is_secure") {
        Ok("is_secure")
    } else if names.iter().any(|name| name == "secure") {
        Ok("secure")
    } else {
        Err(CliError::Config(
            "Chromium cookies table has neither is_secure nor secure column".into(),
        ))
    }
}

#[cfg(windows)]
struct WindowsChromiumDecryptor {
    key: Option<Vec<u8>>,
    meta_version: i64,
}

#[cfg(windows)]
impl WindowsChromiumDecryptor {
    fn new(browser_root: &Path, meta_version: i64) -> Result<Self, CliError> {
        let key = windows_v10_key(browser_root)?;
        Ok(Self { key, meta_version })
    }

    fn decrypt(&self, encrypted: &[u8]) -> Result<Option<String>, CliError> {
        if encrypted.starts_with(b"v10") {
            let Some(key) = self.key.as_deref() else {
                return Ok(None);
            };
            if encrypted.len() < 3 + 12 + 16 {
                return Ok(None);
            }
            let nonce = &encrypted[3..15];
            let sealed = &encrypted[15..];
            let cipher = Aes256Gcm::new_from_slice(key)
                .map_err(|_| CliError::Config("invalid Chromium AES key length".into()))?;
            let Ok(plaintext) = cipher.decrypt(Nonce::from_slice(nonce), sealed) else {
                return Ok(None);
            };
            return decode_chromium_plaintext(plaintext, self.meta_version);
        }

        if encrypted.starts_with(b"v20") {
            // Chromium app-bound encryption is detected explicitly rather than
            // being misinterpreted as legacy DPAPI data.
            return Ok(None);
        }

        match dpapi_unprotect(encrypted) {
            Ok(plaintext) => decode_chromium_plaintext(plaintext, self.meta_version),
            Err(_) => Ok(None),
        }
    }
}

#[cfg(windows)]
fn windows_v10_key(browser_root: &Path) -> Result<Option<Vec<u8>>, CliError> {
    let Some(local_state) = newest_named_file(browser_root, "Local State", 2) else {
        return Ok(None);
    };
    let data = fs::read_to_string(&local_state)?;
    let json: serde_json::Value = serde_json::from_str(&data)?;
    let Some(encoded) = json
        .get("os_crypt")
        .and_then(|value| value.get("encrypted_key"))
        .and_then(|value| value.as_str())
    else {
        return Ok(None);
    };
    let encrypted = BASE64
        .decode(encoded)
        .map_err(|e| CliError::Config(format!("invalid Chromium encrypted_key: {e}")))?;
    let Some(ciphertext) = encrypted.strip_prefix(b"DPAPI") else {
        return Ok(None);
    };
    Ok(Some(dpapi_unprotect(ciphertext)?))
}

#[cfg(windows)]
fn decode_chromium_plaintext(
    mut plaintext: Vec<u8>,
    meta_version: i64,
) -> Result<Option<String>, CliError> {
    if meta_version >= 24 {
        if plaintext.len() < 32 {
            return Ok(None);
        }
        plaintext.drain(..32);
    }
    match String::from_utf8(plaintext) {
        Ok(value) => Ok(Some(value)),
        Err(_) => Ok(None),
    }
}

#[cfg(windows)]
fn dpapi_unprotect(ciphertext: &[u8]) -> Result<Vec<u8>, CliError> {
    use std::ptr::{null, null_mut};
    use std::slice;
    use windows_sys::Win32::Security::Cryptography::{CRYPT_INTEGER_BLOB, CryptUnprotectData};
    use windows_sys::Win32::System::Memory::LocalFree;

    let mut input = CRYPT_INTEGER_BLOB {
        cbData: ciphertext.len() as u32,
        pbData: ciphertext.as_ptr() as *mut u8,
    };
    let mut output = CRYPT_INTEGER_BLOB {
        cbData: 0,
        pbData: null_mut(),
    };
    let ok = unsafe {
        CryptUnprotectData(
            &mut input,
            null_mut(),
            null(),
            null_mut(),
            null(),
            0,
            &mut output,
        )
    };
    if ok == 0 {
        return Err(CliError::Config(
            "Windows DPAPI could not decrypt Chromium cookie material for the current user".into(),
        ));
    }
    let result = unsafe { slice::from_raw_parts(output.pbData, output.cbData as usize) }.to_vec();
    unsafe {
        let _ = LocalFree(output.pbData as isize);
    }
    Ok(result)
}

fn extract_firefox(profile: Option<&str>) -> Result<Vec<CookieRecord>, CliError> {
    let roots = firefox_roots(profile)?;
    let database = roots
        .iter()
        .filter_map(|root| newest_named_file(root, "cookies.sqlite", 4))
        .max_by_key(|path| {
            fs::metadata(path)
                .and_then(|metadata| metadata.modified())
                .unwrap_or(SystemTime::UNIX_EPOCH)
        })
        .ok_or_else(|| CliError::Config("could not find Firefox cookies.sqlite".into()))?;
    let conn = open_browser_database(&database)?;
    let mut stmt = conn
        .prepare(
            "SELECT host, name, value FROM moz_cookies \
             WHERE host = 'suno.com' OR host LIKE '%.suno.com'",
        )
        .map_err(|e| CliError::Config(format!("cannot query Firefox cookies: {e}")))?;
    let rows = stmt
        .query_map([], |row| {
            Ok(CookieRecord {
                domain: row.get(0)?,
                name: row.get(1)?,
                value: row.get(2)?,
            })
        })
        .map_err(|e| CliError::Config(format!("cannot read Firefox cookies: {e}")))?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| CliError::Config(format!("cannot decode Firefox cookie row: {e}")))
}

fn firefox_roots(profile: Option<&str>) -> Result<Vec<PathBuf>, CliError> {
    if let Some(profile) = profile {
        let path = Path::new(profile);
        if path.is_absolute() || profile.contains('\\') || profile.contains('/') {
            return Ok(vec![path.to_path_buf()]);
        }
    }

    #[cfg(windows)]
    {
        let roaming = env::var_os("APPDATA")
            .map(PathBuf::from)
            .ok_or_else(|| CliError::Config("APPDATA is not set".into()))?;
        let local = env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .ok_or_else(|| CliError::Config("LOCALAPPDATA is not set".into()))?;
        let mut roots = vec![roaming.join(r"Mozilla\Firefox\Profiles")];
        roots.push(local.join(
            r"Packages\Mozilla.Firefox_n80bbvh6b1yt2\LocalCache\Roaming\Mozilla\Firefox\Profiles",
        ));
        if let Some(profile) = profile {
            roots = roots.into_iter().map(|root| root.join(profile)).collect();
        }
        return Ok(roots);
    }

    #[cfg(not(windows))]
    {
        let home = env::var_os("HOME")
            .map(PathBuf::from)
            .ok_or_else(|| CliError::Config("HOME is not set".into()))?;
        let mut roots = vec![home.join(".mozilla/firefox")];
        if let Some(profile) = profile {
            roots = roots.into_iter().map(|root| root.join(profile)).collect();
        }
        Ok(roots)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_yt_dlp_style_browser_spec() {
        let spec = parse_browser_spec("brave:Default").unwrap();
        assert_eq!(spec.browser, "brave");
        assert_eq!(spec.profile.as_deref(), Some("Default"));
    }

    #[test]
    fn preserves_windows_path_after_first_colon() {
        let spec = parse_browser_spec(r"brave:C:\Users\Admin\BraveProfile").unwrap();
        assert_eq!(spec.browser, "brave");
        assert_eq!(
            spec.profile.as_deref(),
            Some(r"C:\Users\Admin\BraveProfile")
        );
    }

    #[test]
    fn rejects_unknown_browser() {
        assert!(parse_browser_spec("netscape:Default").is_err());
    }

    #[test]
    fn suno_domain_matching_is_strict() {
        assert!(is_suno_domain("suno.com"));
        assert!(is_suno_domain(".auth.suno.com"));
        assert!(!is_suno_domain("evil-suno.com"));
    }

    #[test]
    fn prefers_auth_domain_clerk_cookie() {
        let auth = records_to_browser_auth(
            "test",
            vec![
                CookieRecord {
                    domain: ".suno.com".into(),
                    name: "__client".into(),
                    value: "generic".into(),
                },
                CookieRecord {
                    domain: "auth.suno.com".into(),
                    name: "__client".into(),
                    value: "preferred".into(),
                },
            ],
        )
        .unwrap();
        assert_eq!(auth.clerk_client_cookie, "preferred");
    }
}
