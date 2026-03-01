module tests.test_83;

@Getter
class User_83 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_83() {
    let u = User_83(83, "User_83")
    assert(u.getId() == 83)
}


pub fn main() {
    test_83()
}
