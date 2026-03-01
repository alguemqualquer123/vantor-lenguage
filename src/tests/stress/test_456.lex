module tests.test_456;

@Getter
class User_456 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_456() {
    let u = User_456(456, "User_456")
    assert(u.getId() == 456)
}


pub fn main() {
    test_456()
}
