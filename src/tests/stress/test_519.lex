module tests.test_519;

@Getter
class User_519 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_519() {
    let u = User_519(519, "User_519")
    assert(u.getId() == 519)
}


pub fn main() {
    test_519()
}
