module tests.test_146;

@Getter
class User_146 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_146() {
    let u = User_146(146, "User_146")
    assert(u.getId() == 146)
}


pub fn main() {
    test_146()
}
