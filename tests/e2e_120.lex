// e2e_120 - full integration
struct Item {
    sku: int,
    qty: int
}
enum Flag {
    On,
    Off
}
fn total<T>(x: T) where T: Show -> T {
    return x;
}
fn label(f: Flag) -> i64 {
    match f {
        On => 1,
        _ => 0
    }
    return 0;
}
pub fn main() -> void {
    let it = Item { sku: 7, qty: 3 };
    let t = total(9);
    let l = label(On);
    let m = l > 0 ? 100 : 0;
    if m > 0 {
        let n = m + 1;
    } else {
        let n = 0;
    }
    loop {
        break;
    }
    return;
}
