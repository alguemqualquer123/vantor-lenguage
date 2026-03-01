module tests.test_635;

@Getter
class User_635 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_635() {
    let u = User_635(635, "User_635")
    assert(u.getId() == 635)
}


pub fn main() {
    test_635()
}
