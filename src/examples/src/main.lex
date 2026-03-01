const test1 = 2

/// Multiplies two integers and returns the result.
fn mul(a: int, b: int) -> int {
    return a * b
}

fn exec(num: int) {
    num |> mul(2) |> |v| "Aoba! {v}" |> print()
}

fn main() {
    test1 |> exec()
}

main()