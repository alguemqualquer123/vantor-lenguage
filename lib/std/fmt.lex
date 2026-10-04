// Lexicon Standard Library — fmt.
// Go-parity formatted I/O (plan: Docs/PLANO_GO_FULL_PARITY.md, Fase 1).
//
// Verbs supported: %v %d %f %s %t %x %q and %%. Flag/width/precision
// processing (e.g. "%08.2f") awaits the formatter upgrade tracked in
// the plan (Fase 4, text/template + full fmt). Variadic Go calls take
// a leading Dynamic list: `fmt::Sprintf("%v=%v", [a, b])`.

import std::strconv;
import std::errors;

/// Returns the concatenation of the elements' default string forms.
pub fn Sprint(args: Dynamic) -> String {
    let out = "";
    let i = 0;
    let n = args.len();
    while i < n {
        out = out + args[i].to_string();
        i = i + 1;
    }
    return out;
}

/// Like Sprint but appends a trailing newline.
pub fn Sprintln(args: Dynamic) -> String {
    return Sprint(args) + "\n";
}

/// Formats according to the verb subset and returns the resulting string
/// (Go's `fmt.Sprintf`; args passed as a list).
pub fn Sprintf(format: String, args: Dynamic) -> String {
    let out = "";
    let i = 0;
    let n = format.len();
    let argi = 0;
    let nargs = args.len();
    while i < n {
        let c = format.char_at(i) as String;
        if c != "%" {
            out = out + c;
            i = i + 1;
        } else {
            i = i + 1;
            if i >= n {
                out = out + "%!(NOVERB)";
                break;
            }
            let verb = format.char_at(i) as String;
            i = i + 1;
            if verb == "%" {
                out = out + "%";
            } else {
                if argi >= nargs {
                    out = out + "%!" + verb + "(MISSING)";
                } else {
                    let a = args[argi];
                    argi = argi + 1;
                    if verb == "v" {
                        out = out + a.to_string();
                    } else {
                        if verb == "d" {
                            out = out + a.to_string();
                        } else {
                            if verb == "s" {
                                out = out + a.to_string();
                            } else {
                                if verb == "f" {
                                    out = out + a.to_string();
                                } else {
                                    if verb == "t" {
                                        out = out + a.to_string();
                                    } else {
                                        if verb == "x" {
                                            out = out + strconv::FormatInt(a, 16);
                                        } else {
                                            if verb == "q" {
                                                out = out + strconv::Quote(a.to_string());
                                            } else {
                                                out = out + "%!" + verb + "(BADVERB)";
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    return out;
}

/// Writes the concatenation of the elements to standard output
/// (Go's `fmt.Print`).
pub fn Print(args: Dynamic) -> void {
    Console::write(Sprint(args));
}

/// Writes the elements followed by a newline (Go's `fmt.Println`).
pub fn Println(args: Dynamic) -> void {
    Console::log(Sprint(args));
}

/// Writes the formatted string (Go's `fmt.Printf`).
pub fn Printf(format: String, args: Dynamic) -> void {
    Console::write(Sprintf(format, args));
}

/// Formats and wraps as an error (Go's `fmt.Errorf`).
pub fn Errorf(format: String, args: Dynamic) -> Dynamic {
    return errors::New(Sprintf(format, args));
}
