module tests.test_283;

@Getter
class User_283 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_283() {
    let u = User_283(283, "User_283")
    assert(u.getId() == 283)
}


pub fn main() {
    test_283()
}
