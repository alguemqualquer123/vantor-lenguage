// Lexicon Standard Library — encoding/pem.
// Go-parity PEM blocks (plan: Docs/PLANO_GO_FULL_PARITY.md, Fase 4).
// Base64 payloads decode through `std::encoding::base64` (ASCII fast
// path). Import as `import std::encoding::pem;`.

import std::encoding::base64;

/// A PEM block: `typ` (`CERTIFICATE`, …) plus raw `bytes` (Go's
/// `pem.Block`, bytes list-adapted).
pub struct Block {
    typ: String,
    bytes: Dynamic,
}

/// Decode result plus `ok` (Go's `(block, rest)` pair, error-as-`ok`).
pub struct DecodeResult {
    block: Block,
    rest: String,
    ok: bool,
}

/// Decodes the first `-----BEGIN …-----` block (Go's `pem.Decode`).
pub fn Decode(s: String) -> DecodeResult {
    let fail = DecodeResult { block: Block { typ: "", bytes: [] }, rest: s, ok: false };
    let bi = Text::index_of(s, "-----BEGIN ");
    if bi < 0 {
        return fail;
    }
    let tail = Text::slice(s, bi + 11, s.len());
    let he = Text::index_of(tail, "-----");
    if he < 0 {
        return fail;
    }
    let typ = Text::slice(tail, 0, he).trim();
    let after = Text::slice(tail, he + 5, tail.len());
    let trailer = "-----END " + typ + "-----";
    let ei = Text::index_of(after, trailer);
    if ei < 0 {
        return fail;
    }
    let payload = Text::slice(after, 0, ei);
    let rest = Text::slice(after, ei + trailer.len(), after.len());
    let clean = "";
    let i = 0;
    while i < payload.len() {
        let c = Text::slice(payload, i, i + 1);
        if c != " " && c != "\t" && c != "\r" && c != "\n" {
            clean = clean + c;
        }
        i = i + 1;
    }
    let d = base64::Decode(clean);
    if !d.ok {
        return DecodeResult { block: Block { typ: typ, bytes: [] }, rest: rest, ok: false };
    }
    let bytes = [];
    let j = 0;
    while j < d.text.len() {
        bytes.push(Text::code_at(d.text, j));
        j = j + 1;
    }
    return DecodeResult { block: Block { typ: typ, bytes: bytes }, rest: rest, ok: true };
}

/// Encodes a block (Go's `pem.EncodeToMemory`, string-adapted).
pub fn Encode(typ: String, data: String) -> String {
    let b64 = base64::Encode(data);
    let out = "-----BEGIN " + typ + "-----\n";
    let i = 0;
    while i < b64.len() {
        out = out + Text::slice(b64, i, i + 64) + "\n";
        i = i + 64;
    }
    return out + "-----END " + typ + "-----\n";
}
