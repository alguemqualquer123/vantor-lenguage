// e2e_125 - Go-parity stdlib: maps, cmp, iter, lambda higher-order fns
module e2e_125;
import std::maps;
import std::cmp;
import std::iter;
import std::slices;

pub fn main() -> void {
    // maps (pair-list maps, functional updates)
    let m = maps::New();
    m = maps::Set(m, "a", 1);
    m = maps::Set(m, "b", 2);
    assert(maps::Len(m) == 2, "Len");
    assert(maps::Get(m, "a").value == 1, "Get");
    assert(maps::Has(m, "b"), "Has");
    assert(!maps::Has(m, "z"), "Has miss");
    m = maps::Delete(m, "a");
    assert(maps::Len(m) == 1, "Delete");
    assert(maps::Keys(m)[0] == "b", "Keys");
    assert(maps::Values(m)[0] == 2, "Values");
    let m2 = maps::Set(maps::New(), "b", 2);
    assert(maps::Equal(m, m2), "Equal");

    // cmp
    assert(cmp::CompareInt(1, 2) == -1, "CompareInt");
    assert(cmp::CompareString("b", "a") == 1, "CompareString");
    assert(cmp::Less(1, 2), "Less");
    assert(cmp::Or([0, 0, 5]) == 5, "Or");

    // lambdas are first-class values
    let double = |x| x * 2;
    assert(double(21) == 42, "lambda call");
    let k = 10;
    let addk = |x| x + k;
    assert(addk(5) == 15, "lambda capture");

    // iter adapters over lambdas
    assert(slices::Equal(iter::Map([1, 2, 3], |x| x * 2), [2, 4, 6]), "iter Map");
    assert(slices::Equal(iter::Filter([1, 2, 3, 4], |x| x > 2), [3, 4]), "iter Filter");
    assert(iter::Reduce([1, 2, 3], 0, |a, x| a + x) == 6, "iter Reduce");
    assert(slices::Equal(iter::Take([1, 2, 3], 2), [1, 2]), "iter Take");
    assert(slices::Equal(slices::Map([1, 2], |x| x + 10), [11, 12]), "slices Map");
    assert(slices::Equal(slices::SortFunc([3, 1, 2], |a, b| a < b), [1, 2, 3]), "SortFunc");
    return;
}
