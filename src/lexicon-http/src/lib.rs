pub mod module;
pub mod response;

pub use module::HttpModule;
pub use response::JsonResponse as ResponseObj;
pub use response::{Response, response_ok, response_err};

pub fn int_to_string(n: i32) -> String {
    if n == 0 {
        return "0".to_string();
    }
    if n < 0 {
        return "-".to_string() + &int_to_string_pos(-n);
    }
    int_to_string_pos(n)
}

fn int_to_string_pos(n: i32) -> String {
    if n == 0 {
        return "".to_string();
    }
    let digit = n % 10;
    let rest = n / 10;
    int_to_string_pos(rest) + &digit_char(digit)
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
