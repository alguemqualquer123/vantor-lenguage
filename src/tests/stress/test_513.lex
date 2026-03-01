module tests.test_513;

@Getter
class User_513 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_513() {
    let u = User_513(513, "User_513")
    assert(u.getId() == 513)
}


pub fn main() {
    test_513()
}
