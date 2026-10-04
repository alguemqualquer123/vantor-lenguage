// Unicode graphemes, normalization and regex with DoS limits
// (Spec §9 + §52).
//
// Grapheme segmentation follows a simplified UAX #29: base character +
// combining marks (Mn/Mc/Me ranges), ZWJ sequences and CR/LF pairs stay
// together. Full UAX tables are an integration point; the API is stable.

/// True for combining marks (Mn, Mc, Me approximations).
fn is_combining(c: char) -> bool {
    matches!(c,
        '\u{0300}'..='\u{036F}' | '\u{0483}'..='\u{0489}' |
        '\u{0591}'..='\u{05BD}' | '\u{05BF}' | '\u{05C1}'..='\u{05C2}' |
        '\u{05C4}'..='\u{05C5}' | '\u{05C7}' | '\u{0610}'..='\u{061A}' |
        '\u{0646}'..='\u{065F}' | '\u{0670}' | '\u{06D6}'..='\u{06DC}' |
        '\u{20D0}'..='\u{20FF}' | '\u{FE20}'..='\u{FE2F}')
}

/// Split into extended-grapheme-like clusters.
pub fn graphemes(s: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut chars = s.chars().peekable();
    let mut prev_was_zwj = false;
    while let Some(c) = chars.next() {
        if c == '\u{200D}' {
            // Zero-width joiner glues to the previous cluster.
            if let Some(last) = out.last_mut() {
                last.push(c);
                prev_was_zwj = true;
                continue;
            }
        }
        if c == '\n' {
            if let Some(last) = out.last() {
                if last == "\r" {
                    out.pop();
                    out.push("\r\n".to_string());
                    prev_was_zwj = false;
                    continue;
                }
            }
        }
        if prev_was_zwj || (is_combining(c) && !out.is_empty()) {
            out.last_mut().unwrap().push(c);
            prev_was_zwj = false;
        } else {
            out.push(c.to_string());
            prev_was_zwj = false;
        }
    }
    out
}

pub fn grapheme_count(s: &str) -> usize {
    graphemes(s).len()
}

/// Minimal NFC-ish composition for common Latin letters with combining
/// marks (e + ́ → é, etc.). Anything outside the table passes through;
/// full UCD normalization tables are an integration point.
pub fn normalize_nfc(s: &str) -> String {
    let clusters = graphemes(s);
    let mut out = String::new();
    for g in clusters {
        out.push_str(&compose_cluster(&g));
    }
    out
}

fn compose_cluster(g: &str) -> String {
    let mut chars = g.chars();
    let base = match chars.next() {
        Some(c) => c,
        None => return String::new(),
    };
    let rest: String = chars.collect();
    if rest.chars().count() != 1 {
        return g.to_string();
    }
    let mark = rest.chars().next().unwrap();
    let composed = match (base, mark) {
        ('a', '\u{0301}') => 'á', ('a', '\u{0300}') => 'à', ('a', '\u{0302}') => 'â',
        ('a', '\u{0303}') => 'ã', ('a', '\u{0308}') => 'ä', ('a', '\u{030A}') => 'å',
        ('e', '\u{0301}') => 'é', ('e', '\u{0300}') => 'è', ('e', '\u{0302}') => 'ê',
        ('e', '\u{0308}') => 'ë',
        ('i', '\u{0301}') => 'í', ('i', '\u{0300}') => 'ì', ('i', '\u{0302}') => 'î',
        ('i', '\u{0308}') => 'ï',
        ('o', '\u{0301}') => 'ó', ('o', '\u{0300}') => 'ò', ('o', '\u{0302}') => 'ô',
        ('o', '\u{0303}') => 'õ', ('o', '\u{0308}') => 'ö',
        ('u', '\u{0301}') => 'ú', ('u', '\u{0300}') => 'ù', ('u', '\u{0302}') => 'û',
        ('u', '\u{0308}') => 'ü',
        ('n', '\u{0303}') => 'ñ', ('c', '\u{0327}') => 'ç',
        ('A', '\u{0301}') => 'Á', ('E', '\u{0301}') => 'É', ('N', '\u{0303}') => 'Ñ',
        _ => return g.to_string(),
    };
    composed.to_string()
}

// ---------------------------------------------------------------------------
// Regex with denial-of-service limits (Spec §52)
// ---------------------------------------------------------------------------

/// Maximum input scanned in one call (1 MiB default).
pub const MAX_SCAN: usize = 1 << 20;
/// Maximum matches returned in one call.
pub const MAX_MATCHES: usize = 10_000;
/// Maximum pattern length accepted (64 KiB). Longer patterns are rejected
/// at compile time to bound program size (DoS defense).
/// Ownership: n/a. Complexity: O(1) check.
pub const MAX_PATTERN_LEN: usize = 64 << 10;

/// Regex wrapper enforcing size limits and match-count caps.
///
/// Ownership: owns the compiled program. Thread-safety: `Send + Sync`
/// (`regex::Regex` is thread-safe); share freely.
/// Complexity: compile O(pattern); match O(input) worst-case linear for
/// DFA-backed patterns (no backtracking engine — catastrophic backtracking
/// class is absent by construction).
pub struct LexRegex {
    inner: regex::Regex,
}

/// Spec §52 alias: `Regex` is the stdlib-facing name.
#[allow(dead_code)]
pub type Regex = LexRegex;

impl LexRegex {
    /// Compile with a bounded in-memory program (`size_limit` 10 MiB).
    /// Rejects patterns over [`MAX_PATTERN_LEN`] (DoS defense).
    /// Ownership: returns owned handle. Complexity: O(pattern).
    pub fn new(pattern: &str) -> Result<Self, String> {
        if pattern.len() > MAX_PATTERN_LEN {
            return Err(format!("E0102: regex pattern exceeds {} bytes", MAX_PATTERN_LEN));
        }
        let inner = regex::RegexBuilder::new(pattern)
            .size_limit(10 << 20)
            .dfa_size_limit(10 << 20)
            .build()
            .map_err(|e| format!("E0102: invalid regex: {}", e))?;
        Ok(LexRegex { inner })
    }

    /// Pattern length guard helper. Complexity: O(1).
    pub fn check_budget(text: &str) -> Result<(), String> {
        if text.len() > MAX_SCAN {
            return Err(format!("E0601: regex input exceeds {} bytes", MAX_SCAN));
        }
        Ok(())
    }

    pub fn is_match(&self, text: &str) -> Result<bool, String> {
        if text.len() > MAX_SCAN {
            return Err(format!("E0601: regex input exceeds {} bytes", MAX_SCAN));
        }
        Ok(self.inner.is_match(text))
    }

    /// Simplified timeout: runs the match then rejects when wall-clock
    /// exceeds `max_millis`. This bounds *observed* latency (DoS triage);
    /// it does not preempt the engine mid-scan — pair with [`MAX_SCAN`] /
    /// [`MAX_MATCHES`] caps for hard guarantees.
    /// Ownership: borrows self + text. Thread-safety: safe to share.
    /// Complexity: O(input) + O(1) clock reads.
    pub fn is_match_timeout(&self, text: &str, max_millis: u64) -> Result<bool, String> {
        Self::check_budget(text)?;
        let t = std::time::Instant::now();
        let out = self.inner.is_match(text);
        if t.elapsed().as_millis() > u128::from(max_millis) {
            return Err(format!("E0601: regex timeout after {} ms", max_millis));
        }
        Ok(out)
    }

    pub fn find_all(&self, text: &str) -> Result<Vec<(usize, usize)>, String> {
        if text.len() > MAX_SCAN {
            return Err(format!("E0601: regex input exceeds {} bytes", MAX_SCAN));
        }
        let mut out = Vec::new();
        for m in self.inner.find_iter(text) {
            out.push((m.start(), m.end()));
            if out.len() > MAX_MATCHES {
                return Err(format!("E0601: regex match cap {} exceeded", MAX_MATCHES));
            }
        }
        Ok(out)
    }

    /// `find_all` with the same simplified wall-clock timeout as
    /// [`LexRegex::is_match_timeout`]. Complexity: O(input).
    pub fn find_all_timeout(&self, text: &str, max_millis: u64) -> Result<Vec<(usize, usize)>, String> {
        Self::check_budget(text)?;
        let t = std::time::Instant::now();
        let mut out = Vec::new();
        for m in self.inner.find_iter(text) {
            out.push((m.start(), m.end()));
            if out.len() > MAX_MATCHES {
                return Err(format!("E0601: regex match cap {} exceeded", MAX_MATCHES));
            }
            if t.elapsed().as_millis() > u128::from(max_millis) {
                return Err(format!("E0601: regex timeout after {} ms", max_millis));
            }
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn combining_marks_cluster() {
        // e + combining acute = one grapheme.
        assert_eq!(grapheme_count("é"), 1);
        assert_eq!(grapheme_count("hello"), 5);
        assert_eq!(grapheme_count("a\r\nb"), 3);
    }

    #[test]
    fn nfc_composes_latin() {
        assert_eq!(normalize_nfc("é"), "é");
        assert_eq!(normalize_nfc("plain"), "plain");
    }

    #[test]
    fn regex_limits() {
        let re = LexRegex::new(r"\d+").unwrap();
        assert!(re.is_match("abc123").unwrap());
        assert_eq!(re.find_all("a1b22").unwrap(), vec![(1, 2), (3, 5)]);
        assert!(LexRegex::new("(").is_err());
        assert!(re.is_match(&"x".repeat(MAX_SCAN + 1)).is_err());
    }

    #[test]
    fn regex_timeout_and_pattern_cap() {
        let re = LexRegex::new(r"\d+").unwrap();
        assert!(re.is_match_timeout("abc123", 1000).unwrap());
        assert_eq!(re.find_all_timeout("a1b22", 1000).unwrap(), vec![(1, 2), (3, 5)]);
        assert!(re.is_match_timeout(&"x".repeat(MAX_SCAN + 1), 1000).is_err());
        assert!(LexRegex::new(&"a".repeat(MAX_PATTERN_LEN + 1)).is_err());
    }
}
