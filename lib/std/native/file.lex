// Lexicon SDK — native signature stubs (File).
// Documentation for IDE navigation ONLY (see native/console.lex header).

/// Reads a whole file (aborts when unreadable).
pub fn read_string(path: String) -> String {
    panic("native stub");
}

/// Writes a whole file; reports success.
pub fn write_string(path: String, data: String) -> bool {
    panic("native stub");
}

/// Reports whether the path exists.
pub fn exists(path: String) -> bool {
    panic("native stub");
}

/// Reports whether the path is a directory.
pub fn is_dir(path: String) -> bool {
    panic("native stub");
}

/// Reports whether the path is a file.
pub fn is_file(path: String) -> bool {
    panic("native stub");
}

/// Removes a file.
pub fn remove(path: String) -> bool {
    panic("native stub");
}

/// Removes an (empty) directory.
pub fn remove_dir(path: String) -> bool {
    panic("native stub");
}

/// Creates one directory level.
pub fn mkdir(path: String) -> bool {
    panic("native stub");
}

/// Creates a directory and parents.
pub fn mkdir_all(path: String) -> bool {
    panic("native stub");
}

/// Lists entry names of a directory.
pub fn read_dir(path: String) -> Dynamic {
    panic("native stub");
}

/// Byte size, or -1 when missing.
pub fn size(path: String) -> i64 {
    panic("native stub");
}

/// Current working directory.
pub fn getwd() -> String {
    panic("native stub");
}

/// OS temporary directory.
pub fn temp_dir() -> String {
    panic("native stub");
}

/// Current user home.
pub fn home_dir() -> String {
    panic("native stub");
}
