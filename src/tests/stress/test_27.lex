module tests.test_27;

@Getter
class User_27 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_27() {
    let u = User_27(27, "User_27")
    assert(u.getId() == 27)
}


pub fn main() {
    test_27()
}
