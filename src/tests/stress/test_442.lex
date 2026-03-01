module tests.test_442;

@Getter
class User_442 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_442() {
    let u = User_442(442, "User_442")
    assert(u.getId() == 442)
}


pub fn main() {
    test_442()
}
