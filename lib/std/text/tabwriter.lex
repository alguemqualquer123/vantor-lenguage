// Lexicon Standard Library — text/tabwriter.
// Go-parity column-aligned text (plan: Docs/PLANO_GO_FULL_PARITY.md,
// Fase 4). Cells in a line are separated by '\t'; the line's final cell is
// written verbatim (Go's trailing-tab rule), and `Flush` pads every column
// to its widest cell. Import as `import std::text::tabwriter;`.

/// A pending table: rows of cells plus the padding knobs.
pub struct Writer {
    rows: Dynamic,
    minwidth: i64,
    padding: i64,
}

/// New writer: each column is at least `minwidth` wide and gets `padding`
/// extra spaces (Go's `tabwriter.NewWriter(w, minwidth, tabwidth, padding,
/// ' ', 0)`; `tabwidth` is ignored because cells are split on every tab).
pub fn New(minwidth: i64, padding: i64) -> Writer {
    return Writer { rows: [], minwidth: minwidth, padding: padding };
}

/// Adds one line (Go's `w.Write([]byte(text))` for a single-line write).
pub fn Write(w: Writer, line: String) -> Writer {
    let cells = [];
    let cur = "";
    let i = 0;
    while i < line.len() {
        let b = Text::code_at(line, i);
        if b == 9 {
            cells.push(cur);
            cur = "";
        } else {
            cur = cur + Text::from_code(b);
        }
        i = i + 1;
    }
    cells.push(cur);
    let rows = w.rows;
    rows.push(cells);
    return Writer { rows: rows, minwidth: w.minwidth, padding: w.padding };
}

/// Renders the table and empties the writer (Go's `w.Flush`).
pub fn Flush(w: Writer) -> String {
    let rows = w.rows;
    let n = rows.len();
    // Column count = longest row minus the trailing free cell.
    let cols = 0;
    let i = 0;
    while i < n {
        let row = rows[i];
        if row.len() - 1 > cols {
            cols = row.len() - 1;
        }
        i = i + 1;
    }
    let widths = [];
    let c = 0;
    while c < cols {
        let best = w.minwidth;
        i = 0;
        while i < n {
            let row = rows[i];
            if c < row.len() - 1 {
                let cell = row[c];
                if cell.len() > best {
                    best = cell.len();
                }
            }
            i = i + 1;
        }
        widths.push(best + w.padding);
        c = c + 1;
    }
    let out = "";
    i = 0;
    while i < n {
        let row = rows[i];
        let last = row.len() - 1;
        c = 0;
        while c < last {
            let cell = row[c];
            out = out + cell;
            let pad = widths[c] - cell.len();
            let k = 0;
            while k < pad {
                out = out + " ";
                k = k + 1;
            }
            c = c + 1;
        }
        out = out + row[last];
        out = out + "\n";
        i = i + 1;
    }
    return out;
}

/// Number of buffered rows (Go has no equivalent; useful in tests).
pub fn Lines(w: Writer) -> i64 {
    let rows = w.rows;
    return rows.len();
}
