// Fase 1 stdlib test — strconv, math, sort, slices, errors, path, fmt.
// Run: lex run --ci src/examples/tests/stdlib_fase1.lex

import std::strings;
import std::strconv;
import std::math;
import std::sort;
import std::slices;
import std::errors;
import std::path;
import std::fmt;

fn main() {
    // ---- strconv ----
    Console::log(strconv::Itoa(-12345));
    Console::log(strconv::FormatInt(255, 16));
    let pi = strconv::ParseInt("0", 10);
    Console::log(pi.ok);
    let r = strconv::Atoi("-987");
    Console::log(r.value);
    Console::log(r.ok);
    let bad = strconv::Atoi("12a");
    Console::log(bad.ok);
    Console::log(strconv::FormatBool(true));
    Console::log(strconv::ParseBool("TRUE").value);
    let pf = strconv::ParseFloat("-12.5e2");
    Console::log(pf.ok);
    Console::log(pf.value);
    Console::log(strconv::Quote("hi \"there\"\n"));

    // ---- math ----
    Console::log(math::Abs(0.0 - 3.5));
    Console::log(math::AbsInt(-9));
    Console::log(math::Floor(3.7));
    Console::log(math::Floor(-3.2));
    Console::log(math::Ceil(3.2));
    Console::log(math::Ceil(-3.7));
    Console::log(math::Round(2.5));
    Console::log(math::Round(-2.5));
    Console::log(math::Trunc(-3.9));
    Console::log(math::Sqrt(2.0));
    Console::log(math::Round(math::Pow(2.0, 10.0)));
    Console::log(math::Round(math::Exp(1.0) * 1000.0));
    Console::log(math::Round(math::Ln(10.0) * 1000.0));
    Console::log(math::Round(math::Log2(1024.0)));
    Console::log(math::Round(math::Log10(1000.0)));
    Console::log(math::Round(math::Cbrt(27.0)));
    Console::log(math::Max(3.5, 7.25));
    Console::log(math::MinInt(4, -2));
    Console::log(math::Clamp(15.0, 0.0, 10.0));
    Console::log(math::Sign(-0.5));

    // ---- sort (returns a new sorted list; Lex lists pass by copy) ----
    let xs = [5, 2, 8, 1, 9, 3, 7, 4, 6, 0, 15, 11];
    xs = sort::Ints(xs);
    Console::log(xs);
    Console::log(sort::IntsAreSorted(xs));
    Console::log(sort::SearchInts(xs, 7));
    Console::log(sort::Floats([2.5, 0.5, 1.5]));
    Console::log(sort::Strings(["pear", "apple", "fig"]));

    // ---- slices ----
    Console::log(slices::Index([10, 20, 30], 20));
    Console::log(slices::Contains([10, 20, 30], 40));
    Console::log(slices::Reverse([1, 2, 3, 4]));
    Console::log(slices::Clone([7, 8]));
    Console::log(slices::Min([3, 1, 2]));
    Console::log(slices::Max([3, 1, 2]));
    Console::log(slices::BinarySearch([1, 3, 5, 7], 5));
    Console::log(slices::Insert([1, 2, 3], 1, 99));
    Console::log(slices::Delete([1, 2, 3, 4], 1, 3));
    Console::log(slices::Replace([1, 2, 3, 4], 1, 3, 9));
    Console::log(slices::Equal([1, 2], [1, 2]));
    Console::log(slices::Compare([1, 2], [1, 3]));
    Console::log(slices::Compact([1, 1, 2, 2, 3, 3]));

    // ---- errors ----
    let err = errors::New("boom");
    Console::log(errors::Message(err));
    Console::log(errors::Is(err, errors::New("boom")));
    Console::log(errors::Is(err, errors::New("other")));

    // ---- path ----
    Console::log(path::Base("/usr/local/bin/go.exe"));
    Console::log(path::Dir("/usr/local/bin/go.exe"));
    Console::log(path::Ext("/usr/local/bin/go.exe"));
    Console::log(path::Join(["/usr", "local", "bin"]));
    Console::log(path::Clean("/usr//local/./bin/../lib"));
    Console::log(path::IsAbs("/usr"));
    Console::log(path::Split("/usr/local"));

    // ---- fmt ----
    Console::log(fmt::Sprintf("%v + %v = %v", [2, 3, 5]));
    Console::log(fmt::Sprintf("hex %x quoted %q bool %t", [255, "a\nb", false]));
    Console::log(fmt::Sprintf("bad %z and missing %d", [1]));
    Console::log(fmt::Sprint(["con", "cat"]));
}
