module tests.test_983;

@Getter
class User_983 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_983() {
    let u = User_983(983, "User_983")
    assert(u.getId() == 983)
}


pub fn main() {
    test_983()
}
