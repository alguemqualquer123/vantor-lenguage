module tests.test_392;

@Getter
class User_392 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_392() {
    let u = User_392(392, "User_392")
    assert(u.getId() == 392)
}


pub fn main() {
    test_392()
}
