// csv_sum.lex — parse a CSV column with std::encoding::csv.
import std::encoding::csv;
import std::strconv;

pub fn main() -> void {
    let doc = "name,score\nada,97\ngrace,100\n";
    let rows = csv::ReadAll(doc);
    let total = 0;
    let i = 1;
    while i < rows.len() {
        total = total + strconv::Atoi(rows[i][1]).value;
        i = i + 1;
    }
    Console::log("total=" + total.to_string());
}
