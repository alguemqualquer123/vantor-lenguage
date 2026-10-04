// e2e_121 - Go-parity stdlib: strconv, math, errors, path, fmt
module e2e_121;
import std::strconv;
import std::math;
import std::errors;
import std::path;
import std::fmt;

pub fn main() -> void {
    // strconv
    assert(strconv::Itoa(-12345) == "-12345", "Itoa");
    assert(strconv::FormatInt(255, 16) == "ff", "FormatInt");
    let ai = strconv::Atoi("-987");
    assert(ai.value == -987, "Atoi value");
    assert(ai.ok, "Atoi ok");
    let abad = strconv::Atoi("12a");
    assert(!abad.ok, "Atoi bad");
    let p36 = strconv::ParseInt("z", 36);
    assert(p36.value == 35, "ParseInt base36");
    assert(strconv::ParseBool("TRUE").value, "ParseBool");
    let pf = strconv::ParseFloat("-12.5e2");
    assert(pf.ok, "ParseFloat ok");
    assert(pf.value == -1250.0, "ParseFloat value");
    assert(strconv::FormatBool(false) == "false", "FormatBool");
    assert(strconv::Quote("a\nb") == "\"a\\nb\"", "Quote");

    // math
    assert(math::Abs(0.0 - 3.5) == 3.5, "Abs");
    assert(math::AbsInt(-9) == 9, "AbsInt");
    assert(math::Floor(3.7) == 3.0, "Floor");
    assert(math::Floor(-3.2) == -4.0, "Floor neg");
    assert(math::Ceil(3.2) == 4.0, "Ceil");
    assert(math::Ceil(-3.7) == -3.0, "Ceil neg");
    assert(math::Round(2.5) == 3.0, "Round");
    assert(math::Round(-2.5) == -3.0, "Round neg");
    assert(math::Trunc(-3.9) == -3.0, "Trunc");
    assert(math::Sqrt(16.0) == 4.0, "Sqrt");
    assert(math::Round(math::Pow(2.0, 10.0)) == 1024.0, "Pow");
    assert(math::Round(math::Exp(1.0) * 1000.0) == 2718.0, "Exp");
    assert(math::Round(math::Ln(10.0) * 1000.0) == 2303.0, "Ln");
    assert(math::Round(math::Log2(1024.0)) == 10.0, "Log2");
    assert(math::Round(math::Log10(1000.0)) == 3.0, "Log10");
    assert(math::Round(math::Cbrt(27.0)) == 3.0, "Cbrt");
    assert(math::Max(3.5, 7.25) == 7.25, "Max");
    assert(math::MinInt(4, -2) == -2, "MinInt");
    assert(math::MaxInt(4, -2) == 4, "MaxInt");
    assert(math::Clamp(15.0, 0.0, 10.0) == 10.0, "Clamp");
    assert(math::Sign(-0.5) == -1, "Sign");

    // errors
    let err = errors::New("boom");
    assert(errors::Message(err) == "boom", "errors Message");
    assert(errors::Is(err, errors::New("boom")), "errors Is");
    assert(!errors::Is(err, errors::New("other")), "errors Is not");

    // path
    assert(path::Base("/usr/local/bin/go.exe") == "go.exe", "path Base");
    assert(path::Dir("/usr/local/bin/go.exe") == "/usr/local/bin", "path Dir");
    assert(path::Ext("/usr/local/bin/go.exe") == ".exe", "path Ext");
    assert(path::Join(["/usr", "local", "bin"]) == "/usr/local/bin", "path Join");
    assert(path::Clean("/usr//local/./bin/../lib") == "/usr/local/lib", "path Clean");
    assert(path::IsAbs("/usr"), "path IsAbs");
    assert(!path::IsAbs("usr"), "path not IsAbs");
    let sp = path::Split("/usr/local");
    assert(sp[0] == "/usr/", "path Split dir");
    assert(sp[1] == "local", "path Split file");

    // fmt
    assert(fmt::Sprintf("%v + %v = %v", [2, 3, 5]) == "2 + 3 = 5", "Sprintf v");
    assert(fmt::Sprintf("hex %x", [255]) == "hex ff", "Sprintf x");
    assert(fmt::Sprintf("%t", [true]) == "true", "Sprintf t");
    assert(fmt::Sprint(["con", "cat"]) == "concat", "Sprint");
    return;
}
