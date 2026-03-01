module tests.test_548;

@Getter
class User_548 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_548() {
    let u = User_548(548, "User_548")
    assert(u.getId() == 548)
}


pub fn main() {
    test_548()
}
