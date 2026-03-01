module tests.test_407;

@Getter
class User_407 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_407() {
    let u = User_407(407, "User_407")
    assert(u.getId() == 407)
}


pub fn main() {
    test_407()
}
