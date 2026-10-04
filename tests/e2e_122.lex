// e2e_122 - Go-parity stdlib: strings, sort, slices
module e2e_122;
import std::strings;
import std::sort;
import std::slices;

pub fn main() -> void {
    // strings
    assert(strings::ToUpper("hello lexicon") == "HELLO LEXICON", "ToUpper");
    assert(strings::ToLower("HELLO") == "hello", "ToLower");
    assert(strings::HasPrefix("hello world", "hello"), "HasPrefix");
    assert(!strings::HasPrefix("hello", "world"), "HasPrefix neg");
    assert(strings::HasSuffix("hello world", "world"), "HasSuffix");
    assert(strings::Join(["a", "b", "c"], "-") == "a-b-c", "Join");
    assert(strings::Repeat("ab", 3) == "ababab", "Repeat");
    assert(strings::Index("hello", "ll") == 2, "Index");
    assert(strings::Index("hello", "z") == -1, "Index miss");
    assert(strings::TrimSpace("  padded  ") == "padded", "TrimSpace");
    assert(strings::ReplaceAll("a-b-c", "-", "+") == "a+b+c", "ReplaceAll");
    let parts = strings::Split("x,y,z", ",");
    assert(parts.len() == 3, "Split len");
    assert(parts[1] == "y", "Split elem");
    assert(strings::Contains("hello", "ell"), "Contains");
    assert(!strings::Contains("hello", "z"), "Contains neg");
    assert(strings::RuneCount("lexicon") == 7, "RuneCount");

    // sort (functions return a new sorted list; Lex lists pass by copy)
    let xs = [5, 2, 8, 1, 9, 3, 7, 4, 6, 0, 15, 11];
    xs = sort::Ints(xs);
    assert(slices::Equal(xs, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 11, 15]), "sort Ints");
    assert(sort::IntsAreSorted(xs), "IntsAreSorted");
    assert(sort::SearchInts(xs, 7) == 7, "SearchInts found");
    assert(sort::SearchInts(xs, 10) == 10, "SearchInts insertion");
    assert(sort::SearchInts(xs, -1) == 0, "SearchInts front");
    let fs = sort::Floats([2.5, 0.5, 1.5]);
    assert(fs[0] == 0.5 && fs[1] == 1.5 && fs[2] == 2.5, "sort Floats");
    let ss = sort::Strings(["pear", "apple", "fig"]);
    assert(ss[0] == "apple" && ss[1] == "fig" && ss[2] == "pear", "sort Strings");
    assert(sort::FloatsAreSorted(fs), "FloatsAreSorted");
    assert(sort::StringsAreSorted(ss), "StringsAreSorted");

    // slices
    assert(slices::Index([10, 20, 30], 20) == 1, "slices Index");
    assert(slices::Index([10, 20, 30], 40) == -1, "slices Index miss");
    assert(slices::Contains([10, 20, 30], 20), "slices Contains");
    assert(!slices::Contains([10, 20, 30], 40), "slices Contains neg");
    assert(slices::Equal(slices::Reverse([1, 2, 3, 4]), [4, 3, 2, 1]), "Reverse");
    assert(slices::Equal(slices::Clone([7, 8]), [7, 8]), "Clone");
    assert(slices::Min([3, 1, 2]) == 1, "slices Min");
    assert(slices::Max([3, 1, 2]) == 3, "slices Max");
    assert(slices::BinarySearch([1, 3, 5, 7], 5) == 2, "BinarySearch");
    assert(slices::Equal(slices::Insert([1, 2, 3], 1, 99), [1, 99, 2, 3]), "Insert");
    assert(slices::Equal(slices::Delete([1, 2, 3, 4], 1, 3), [1, 4]), "Delete");
    assert(slices::Equal(slices::Replace([1, 2, 3, 4], 1, 3, 9), [1, 9, 4]), "Replace");
    assert(slices::Equal([1, 2], [1, 2]), "Equal");
    assert(!slices::Equal([1, 2], [1, 3]), "Equal neg");
    assert(slices::Compare([1, 2], [1, 3]) == -1, "Compare lt");
    assert(slices::Compare([1, 3], [1, 3]) == 0, "Compare eq");
    assert(slices::Compare([2], [1]) == 1, "Compare gt");
    assert(slices::Equal(slices::Compact([1, 1, 2, 2, 3, 3]), [1, 2, 3]), "Compact");
    return;
}
