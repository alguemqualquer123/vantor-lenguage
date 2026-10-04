// load_recursion_ok — deep but legal call chains (just under MAX_DEPTH).
pub fn down(n: i64) -> i64 {
    if n <= 0 {
        return 0;
    }
    return down(n - 1) + 1;
}

pub fn ping(n: i64) -> i64 {
    if n <= 0 {
        return 0;
    }
    return pong(n - 1) + 1;
}

pub fn pong(n: i64) -> i64 {
    if n <= 0 {
        return 0;
    }
    return ping(n - 1) + 1;
}

pub fn main() -> void {
    assert(down(400) == 400, "depth 400");
    assert(ping(200) == 200, "mutual 200");
    return;
}
