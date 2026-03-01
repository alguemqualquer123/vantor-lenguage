module tests.test_231;

@Getter
class User_231 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_231() {
    let u = User_231(231, "User_231")
    assert(u.getId() == 231)
}


pub fn main() {
    test_231()
}
