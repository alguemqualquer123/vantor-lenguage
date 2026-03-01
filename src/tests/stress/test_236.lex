module tests.test_236;

@Getter
class User_236 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_236() {
    let u = User_236(236, "User_236")
    assert(u.getId() == 236)
}


pub fn main() {
    test_236()
}
