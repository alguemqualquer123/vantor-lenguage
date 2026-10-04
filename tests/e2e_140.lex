// e2e_140 - runtime reflection: typeOf/typeof/type + method forms
module e2e_140;

pub struct User {
    name: String,
}

pub fn main() -> void {
    assert(typeOf(1) == "int", "int");
    assert(typeOf(1.5) == "float", "float");
    assert(typeOf(true) == "bool", "bool");
    assert(typeOf("s") == "String", "String");
    assert(typeOf([1]) == "List", "List");
    assert(typeOf(User { name: "a" }) == "User", "struct name");
    assert(typeof(1) == "int", "typeof alias");
    assert(type(1) == "int", "type alias");
    assert((1).type() == "int", "method type");
    assert("s".typeOf() == "String", "method typeOf");
    return;
}
