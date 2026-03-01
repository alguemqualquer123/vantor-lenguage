module tests.test_819;

@Getter
class User_819 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_819() {
    let u = User_819(819, "User_819")
    assert(u.getId() == 819)
}


pub fn main() {
    test_819()
}
