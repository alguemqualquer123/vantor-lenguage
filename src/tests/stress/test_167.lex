module tests.test_167;

@Getter
class User_167 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_167() {
    let u = User_167(167, "User_167")
    assert(u.getId() == 167)
}


pub fn main() {
    test_167()
}
