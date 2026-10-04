pub fn main() -> void {
    let mut s = "[";
    let mut i: i32 = 0;
    while (i < 1000) {
        if (i > 0) {
            s = s + ",";
        }
        s = s + "{\"id\":0,\"name\":\"item\"}";
        i = i + 1;
    }
    s = s + "]";
    let parsed = Json::parse(s);
    println(parsed);
}
