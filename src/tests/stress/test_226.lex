module tests.test_226;

@Getter
class User_226 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_226() {
    let u = User_226(226, "User_226")
    assert(u.getId() == 226)
}


pub fn main() {
    test_226()
}
