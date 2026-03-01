module tests.test_394;

@Getter
class User_394 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_394() {
    let u = User_394(394, "User_394")
    assert(u.getId() == 394)
}


pub fn main() {
    test_394()
}
