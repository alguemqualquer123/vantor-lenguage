module tests.test_484;

@Getter
class User_484 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_484() {
    let u = User_484(484, "User_484")
    assert(u.getId() == 484)
}


pub fn main() {
    test_484()
}
