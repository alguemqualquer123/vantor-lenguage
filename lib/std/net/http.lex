// Lexicon Standard Library — net/http.
// Go-parity HTTP client surface over the native `Http::*` builtins
// (plan: Docs/PLANO_GO_FULL_PARITY.md, Fase 5 — client first; serving
// stays on `Http::serve` + `@Get` handlers). Import as
// `import std::net::http;`.

/// A client response (Go's `http.Response`, body-first subset: the native
/// fetch returns the body; transport errors surface as empty bodies).
pub struct Response {
    body: String,
    ok: bool,
}

/// Fetches `url` (Go's `http.Get`).
pub fn Get(url: String) -> Response {
    let body = Http::get(url);
    return Response { body: body, ok: true };
}

/// Posts `data` (Go's `http.Post` with an explicit body).
pub fn Post(url: String, data: String) -> Response {
    let body = Http::post(url, data);
    return Response { body: body, ok: true };
}

/// Reads the body (Go's `io.ReadAll(resp.Body)` shape, direct here).
pub fn ReadBody(r: Response) -> String {
    return r.body;
}
