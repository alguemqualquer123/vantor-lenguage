// Lexicon Standard Library — encoding/csv.
// Go-parity CSV (plan: Docs/PLANO_GO_FULL_PARITY.md, Fase 4). RFC 4180
// subset: comma separator, double-quoted fields with `""` escapes,
// `\r\n`/`\n` line breaks. Import as `import std::encoding::csv;`.

/// Parses the whole document into rows of fields (Go's `csv.Reader`
/// `ReadAll`, document-adapted).
pub fn ReadAll(s: String) -> Dynamic {
    let rows = [];
    let row = [];
    let field = "";
    let quoted = false;
    let i = 0;
    let n = s.len();
    while i < n {
        let c = Text::slice(s, i, i + 1);
        if quoted {
            if c == "\"" {
                if i + 1 < n && Text::slice(s, i + 1, i + 2) == "\"" {
                    field = field + "\"";
                    i = i + 2;
                    continue;
                }
                quoted = false;
                i = i + 1;
                continue;
            }
            field = field + c;
            i = i + 1;
            continue;
        }
        if c == "\"" && field == "" {
            quoted = true;
            i = i + 1;
            continue;
        }
        if c == "," {
            row.push(field);
            field = "";
            i = i + 1;
            continue;
        }
        if c == "\r" || c == "\n" {
            row.push(field);
            field = "";
            rows.push(row);
            row = [];
            if c == "\r" && i + 1 < n && Text::slice(s, i + 1, i + 2) == "\n" {
                i = i + 2;
            } else {
                i = i + 1;
            }
            continue;
        }
        field = field + c;
        i = i + 1;
    }
    if quoted {
        return rows;
    }
    if field != "" || row.len() > 0 {
        row.push(field);
        rows.push(row);
    }
    return rows;
}

/// Serializes rows of fields, quoting when needed (Go's `csv.Writer`
/// `WriteAll`, document-adapted).
pub fn WriteAll(rows: Dynamic) -> String {
    let out = "";
    let i = 0;
    while i < rows.len() {
        let row = rows[i];
        let j = 0;
        while j < row.len() {
            if j > 0 {
                out = out + ",";
            }
            out = out + csv_field(row[j].to_string());
            j = j + 1;
        }
        out = out + "\n";
        i = i + 1;
    }
    return out;
}

fn csv_field(s: String) -> String {
    if Text::index_of(s, ",") >= 0 || Text::index_of(s, "\"") >= 0 || Text::index_of(s, "\n") >= 0 {
        return "\"" + s.replace_all("\"", "\"\"") + "\"";
    }
    return s;
}
