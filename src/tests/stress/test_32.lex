module tests.test_32;

@Getter
class User_32 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_32() {
    let u = User_32(32, "User_32")
    assert(u.getId() == 32)
}


pub fn main() {
    test_32()
}
