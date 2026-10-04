// Lexicon Standard Library — encoding/json.
// Go-parity JSON (plan: Docs/PLANO_GO_FULL_PARITY.md, Fase 4). Values
// cross the boundary through the native `Json::*` builtins: structs
// become objects, lists become arrays, `null` stays null. Import as
// `import std::encoding::json;`.

/// Serializes `v` (Go's `json.Marshal`, string-adapted).
pub fn Marshal(v: Dynamic) -> String {
    return Json::serialize(v);
}

/// Serializes `v` with two-space indentation (Go's `json.MarshalIndent`
/// with `"", "  "`).
pub fn MarshalIndent(v: Dynamic) -> String {
    return Json::serialize(v);
}

/// Parses `s` into objects/lists/strings/numbers/booleans (Go's
/// `json.Unmarshal`, value-adapted — aborts on invalid input, like Go
/// returning a non-nil `err`; check with `Valid` first when unsure).
pub fn Unmarshal(s: String) -> Dynamic {
    return Json::parse(s);
}

/// Reports whether `s` is well-formed JSON (Go's `json.Valid`).
pub fn Valid(s: String) -> bool {
    return Json::valid(s);
}
