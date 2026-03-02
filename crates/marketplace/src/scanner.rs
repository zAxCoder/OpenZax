use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

const WASM_MAGIC: &[u8] = &[0x00, 0x61, 0x73, 0x6D];
const WASM_VERSION: &[u8] = &[0x01, 0x00, 0x00, 0x00];

/// Known malicious byte patterns (simplified fingerprints)
const SUSPICIOUS_PATTERNS: &[(&str, &[u8])] = &[
    ("shell_injection_marker", b"\x00sh\x00-c\x00"),
    ("eval_bytecode", b"eval\x00bytecode"),
    ("reverse_shell", b"bash\x20-i\x20>&"),
];

/// Known vulnerable dependency identifiers
const KNOWN_VULNERABLE_DEPS: &[(&str, &str, &str)] = &[
    ("openssl", "< 3.0.7", "CVE-2022-3786"),
    ("log4j", "< 2.17.1", "CVE-2021-44228"),
    ("serde_json", "< 1.0.85", "CVE-2022-xxxx"),
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub passed: bool,
    pub violations: Vec<ScanViolation>,
    pub risk_score: f32,
    pub wasm_size_bytes: usize,
    pub section_count: usize,
}

impl ScanResult {
    fn compute_risk_score(violations: &[ScanViolation]) -> f32 {
        let mut score = 0.0f32;
        for v in violations {
            score += match v.severity {
                ViolationSeverity::Critical => 10.0,
                ViolationSeverity::High => 5.0,
                ViolationSeverity::Medium => 2.0,
                ViolationSeverity::Low => 0.5,
            };
        }
        score.min(100.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanViolation {
    pub severity: ViolationSeverity,
    pub description: String,
    pub line_hint: Option<usize>,
}

impl ScanViolation {
    fn critical(description: impl Into<String>) -> Self {
        Self {
            severity: ViolationSeverity::Critical,
            description: description.into(),
            line_hint: None,
        }
    }

    fn high(description: impl Into<String>) -> Self {
        Self {
            severity: ViolationSeverity::High,
            description: description.into(),
            line_hint: None,
        }
    }

    fn medium(description: impl Into<String>) -> Self {
        Self {
            severity: ViolationSeverity::Medium,
            description: description.into(),
            line_hint: None,
        }
    }

    #[allow(dead_code)]
    fn with_hint(mut self, hint: usize) -> Self {
        self.line_hint = Some(hint);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ViolationSeverity {
    Critical,
    High,
    Medium,
    Low,
}

/// Tier 1: Automated static analysis of WASM packages
pub struct Tier1Scanner;

impl Tier1Scanner {
    pub fn new() -> Self {
        Self
    }

    /// Full scan pipeline: validate bytes, check permissions, detect patterns
    pub fn scan(&self, wasm_bytes: &[u8], manifest_permissions: &[String]) -> ScanResult {
        let mut violations = Vec::new();

        let wasm_check = self.scan_wasm_bytes(wasm_bytes);
        violations.extend(wasm_check.violations.clone());

        let section_count = wasm_check.section_count;

        if wasm_check.passed {
            let perm_violations = self.check_declared_permissions(wasm_bytes, manifest_permissions);
            violations.extend(perm_violations);

            let pattern_violations = self.detect_suspicious_patterns(wasm_bytes);
            violations.extend(pattern_violations);
        }

        let has_critical = violations
            .iter()
            .any(|v| v.severity == ViolationSeverity::Critical);
        let risk_score = ScanResult::compute_risk_score(&violations);

        ScanResult {
            passed: !has_critical && risk_score < 15.0,
            violations,
            risk_score,
            wasm_size_bytes: wasm_bytes.len(),
            section_count,
        }
    }

    /// Validates WASM magic bytes and basic module structure
    pub fn scan_wasm_bytes(&self, bytes: &[u8]) -> ScanResult {
        let mut violations = Vec::new();

        if bytes.len() < 8 {
            violations.push(ScanViolation::critical("WASM binary too small to be valid"));
            return ScanResult {
                passed: false,
                violations,
                risk_score: 100.0,
                wasm_size_bytes: bytes.len(),
                section_count: 0,
            };
        }

        if &bytes[0..4] != WASM_MAGIC {
            violations.push(ScanViolation::critical(
                "Invalid WASM magic bytes - not a WASM module",
            ));
            return ScanResult {
                passed: false,
                violations,
                risk_score: 100.0,
                wasm_size_bytes: bytes.len(),
                section_count: 0,
            };
        }

        if &bytes[4..8] != WASM_VERSION {
            violations.push(ScanViolation::high("Unsupported WASM version"));
        }

        if bytes.len() > 50 * 1024 * 1024 {
            violations.push(ScanViolation {
                severity: ViolationSeverity::Medium,
                description: format!(
                    "WASM binary is very large ({} MB) - may indicate bundled data",
                    bytes.len() / 1024 / 1024
                ),
                line_hint: None,
            });
        }

        let section_count = self.count_wasm_sections(bytes);
        debug!(
            "WASM scan: {} bytes, {} sections",
            bytes.len(),
            section_count
        );

        if section_count == 0 && bytes.len() > 8 {
            violations.push(ScanViolation::medium(
                "WASM module has no recognized sections",
            ));
        }

        let risk_score = ScanResult::compute_risk_score(&violations);
        let passed = violations
            .iter()
            .all(|v| v.severity != ViolationSeverity::Critical);

        ScanResult {
            passed,
            violations,
            risk_score,
            wasm_size_bytes: bytes.len(),
            section_count,
        }
    }

    /// Count WASM sections by iterating the binary format
    fn count_wasm_sections(&self, bytes: &[u8]) -> usize {
        let mut count = 0;
        let mut pos = 8; // Skip magic + version

        while pos < bytes.len() {
            if pos >= bytes.len() {
                break;
            }
            let _section_id = bytes[pos];
            pos += 1;

            // Decode LEB128 section size
            let (size, consumed) = decode_leb128_u32(&bytes[pos..]);
            pos += consumed;
            pos += size as usize;
            count += 1;

            if count > 10_000 {
                break;
            } // Safety limit
        }

        count
    }

    /// Compare declared WIT imports against manifest permissions
    pub fn check_declared_permissions(
        &self,
        wasm_bytes: &[u8],
        manifest_permissions: &[String],
    ) -> Vec<ScanViolation> {
        let mut violations = Vec::new();

        // Extract import names from WASM binary (section type 2 = import section)
        let wasm_imports = self.extract_wasm_imports(wasm_bytes);

        // Check for imports not declared in manifest
        for import in &wasm_imports {
            let declared = manifest_permissions
                .iter()
                .any(|p| import.contains(p.as_str()) || p.contains(import.as_str()));

            if !declared && self.is_sensitive_import(import) {
                violations.push(ScanViolation::high(format!(
                    "Undeclared sensitive import detected: '{import}' not in manifest permissions"
                )));
            }
        }

        // Check for manifest permissions that seem overly broad
        for perm in manifest_permissions {
            if perm == "*" || perm == "all" {
                violations.push(ScanViolation::medium(
                    "Wildcard permission declared - overly broad access requested",
                ));
            }
            if perm.contains("filesystem") && perm.contains("write") {
                violations.push(ScanViolation {
                    severity: ViolationSeverity::Medium,
                    description: "Filesystem write permission requested - ensure this is necessary"
                        .to_string(),
                    line_hint: None,
                });
            }
        }

        violations
    }

    fn extract_wasm_imports(&self, bytes: &[u8]) -> Vec<String> {
        let mut imports = Vec::new();
        let mut pos = 8;

        while pos < bytes.len() {
            if pos >= bytes.len() {
                break;
            }
            let section_id = bytes[pos];
            pos += 1;

            let (size, consumed) = decode_leb128_u32(&bytes[pos..]);
            let section_start = pos + consumed;
            pos = section_start + size as usize;

            if section_id == 2 && section_start < bytes.len() {
                // Import section - parse module/name pairs
                let section_bytes = &bytes[section_start..pos.min(bytes.len())];
                imports.extend(self.parse_import_section(section_bytes));
            }

            if pos > bytes.len() {
                break;
            }
        }

        imports
    }

    fn parse_import_section(&self, section: &[u8]) -> Vec<String> {
        let mut imports = Vec::new();
        let mut pos = 0;

        if pos >= section.len() {
            return imports;
        }
        let (count, consumed) = decode_leb128_u32(&section[pos..]);
        pos += consumed;

        for _ in 0..count.min(1000) {
            if pos >= section.len() {
                break;
            }

            // module name
            let (mod_len, c) = decode_leb128_u32(&section[pos..]);
            pos += c;
            if pos + mod_len as usize > section.len() {
                break;
            }
            let mod_name = std::str::from_utf8(&section[pos..pos + mod_len as usize])
                .unwrap_or("<invalid>")
                .to_string();
            pos += mod_len as usize;

            // field name
            if pos >= section.len() {
                break;
            }
            let (field_len, c) = decode_leb128_u32(&section[pos..]);
            pos += c;
            if pos + field_len as usize > section.len() {
                break;
            }
            let field_name = std::str::from_utf8(&section[pos..pos + field_len as usize])
                .unwrap_or("<invalid>")
                .to_string();
            pos += field_len as usize;

            imports.push(format!("{mod_name}::{field_name}"));

            // Skip import descriptor (at least 1 byte)
            if pos < section.len() {
                pos += 1;
            }
        }

        imports
    }

    fn is_sensitive_import(&self, import: &str) -> bool {
        const SENSITIVE_PREFIXES: &[&str] = &[
            "wasi:filesystem",
            "wasi:sockets",
            "wasi:cli",
            "wasi:http",
            "sys:exec",
            "env:network",
        ];
        SENSITIVE_PREFIXES
            .iter()
            .any(|prefix| import.starts_with(prefix))
    }

    /// Scan for known malicious byte patterns
    pub fn detect_suspicious_patterns(&self, bytes: &[u8]) -> Vec<ScanViolation> {
        let mut violations = Vec::new();

        for (name, pattern) in SUSPICIOUS_PATTERNS {
            if let Some(offset) = find_subsequence(bytes, pattern) {
                warn!(
                    "Suspicious pattern '{}' detected at offset {}",
                    name, offset
                );
                violations.push(ScanViolation {
                    severity: ViolationSeverity::Critical,
                    description: format!("Suspicious pattern detected: {name}"),
                    line_hint: Some(offset),
                });
            }
        }

        // Check for suspiciously high entropy sections (possible encrypted payloads)
        let entropy = compute_byte_entropy(bytes);
        if entropy > 7.8 {
            violations.push(ScanViolation {
                severity: ViolationSeverity::Medium,
                description: format!("High byte entropy ({entropy:.2}/8.0) - possible obfuscation or encrypted payload"),
                line_hint: None,
            });
        }

        violations
    }
}

impl Default for Tier1Scanner {
    fn default() -> Self {
        Self::new()
    }
}

/// Audits declared dependencies against known-vulnerable list
pub struct DependencyAuditor;

impl DependencyAuditor {
    pub fn new() -> Self {
        Self
    }

    pub fn audit_dependencies(
        &self,
        dependencies: &[(String, String)],
    ) -> Vec<DependencyVulnerability> {
        let mut findings = Vec::new();

        for (dep_name, dep_version) in dependencies {
            for (vuln_name, vuln_range, cve) in KNOWN_VULNERABLE_DEPS {
                if dep_name.to_lowercase() == *vuln_name
                    && self.version_matches_range(dep_version, vuln_range)
                {
                    findings.push(DependencyVulnerability {
                        dependency: dep_name.clone(),
                        version: dep_version.clone(),
                        vulnerable_range: vuln_range.to_string(),
                        cve_id: cve.to_string(),
                        severity: ViolationSeverity::High,
                    });
                }
            }
        }

        findings
    }

    fn version_matches_range(&self, version: &str, range: &str) -> bool {
        // Simple range check: supports "< X.Y.Z" format
        if let Some(max_ver) = range.strip_prefix("< ").map(str::trim) {
            return version_less_than(version, max_ver);
        }
        if let Some(max_ver) = range.strip_prefix("<= ").map(str::trim) {
            return version_less_than(version, max_ver) || version == max_ver;
        }
        false
    }
}

impl Default for DependencyAuditor {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyVulnerability {
    pub dependency: String,
    pub version: String,
    pub vulnerable_range: String,
    pub cve_id: String,
    pub severity: ViolationSeverity,
}

// Utilities

fn decode_leb128_u32(bytes: &[u8]) -> (u32, usize) {
    let mut result = 0u32;
    let mut shift = 0u32;
    let mut consumed = 0;

    for &byte in bytes.iter().take(5) {
        consumed += 1;
        result |= ((byte & 0x7F) as u32) << shift;
        if byte & 0x80 == 0 {
            break;
        }
        shift += 7;
    }

    (result, consumed)
}

fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn compute_byte_entropy(data: &[u8]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }

    let mut counts = [0u64; 256];
    for &byte in data {
        counts[byte as usize] += 1;
    }

    let len = data.len() as f64;
    counts
        .iter()
        .filter(|&&c| c > 0)
        .map(|&c| {
            let p = c as f64 / len;
            -p * p.log2()
        })
        .sum()
}

fn version_less_than(a: &str, b: &str) -> bool {
    let parse = |v: &str| -> Vec<u64> { v.split('.').map(|s| s.parse().unwrap_or(0)).collect() };
    let va = parse(a);
    let vb = parse(b);
    va < vb
}
