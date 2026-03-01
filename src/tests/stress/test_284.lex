module tests.test_284;

@Getter
class User_284 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_284() {
    let u = User_284(284, "User_284")
    assert(u.getId() == 284)
}


pub fn main() {
    test_284()
}
