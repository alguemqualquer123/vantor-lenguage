module tests.test_942;

@Getter
class User_942 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_942() {
    let u = User_942(942, "User_942")
    assert(u.getId() == 942)
}


pub fn main() {
    test_942()
}
