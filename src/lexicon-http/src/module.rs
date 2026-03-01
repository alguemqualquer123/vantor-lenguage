pub struct HttpModule;

impl HttpModule {
    pub fn new() -> Self {
        Self
    }
}

pub fn root() -> String {
    r#"{"success":true,"data":"Welcome to LexiconLang API!","message":null}"#.to_string()
}

pub fn hello() -> String {
    r#"{"success":true,"data":"Hello from LexiconLang HTTP Server!","message":null}"#.to_string()
}

pub fn users() -> String {
    r#"{"success":true,"data":[{"id":1,"name":"John","email":"john@example.com"},{"id":2,"name":"Jane","email":"jane@example.com"}],"message":null}"#.to_string()
}

pub fn stats(count: i32) -> String {
    format!(
        r#"{{"success":true,"data":{},"message":"Total requests"}}"#,
        count
    )
}

fn int_to_str(n: i32) -> String {
    if n == 0 {
        return "0".to_string();
    }
    if n < 0 {
        return "-".to_string() + &int_to_str_pos(-n);
    }
    int_to_str_pos(n)
}

fn int_to_str_pos(n: i32) -> String {
    if n == 0 {
        return "".to_string();
    }
    let digit = n % 10;
    let rest = n / 10;
    int_to_str_pos(rest) + &digit_char(digit)
}

fn digit_char(d: i32) -> String {
    match d {
        0 => "0",
        1 => "1",
        2 => "2",
        3 => "3",
        4 => "4",
        5 => "5",
        6 => "6",
        7 => "7",
        8 => "8",
        9 => "9",
        _ => "0",
    }
    .to_string()
}
