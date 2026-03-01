module tests.test_256;

@Getter
class User_256 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_256() {
    let u = User_256(256, "User_256")
    assert(u.getId() == 256)
}


pub fn main() {
    test_256()
}
