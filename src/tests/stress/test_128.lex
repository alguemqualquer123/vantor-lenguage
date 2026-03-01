module tests.test_128;

@Getter
class User_128 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_128() {
    let u = User_128(128, "User_128")
    assert(u.getId() == 128)
}


pub fn main() {
    test_128()
}
