// e2e_100 - enum match with wildcard fallback
enum Suit {
    Spades,
    Hearts,
    Clubs,
    Diamonds
}
fn minor(s: Suit) -> i64 {
    match s {
        Clubs => 1,
        Diamonds => 2,
        _ => 0
    }
    return 0;
}
pub fn main() -> void {
    let r = minor(Hearts);
    return;
}
