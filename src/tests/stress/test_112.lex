module tests.test_112;

@Getter
class User_112 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_112() {
    let u = User_112(112, "User_112")
    assert(u.getId() == 112)
}


pub fn main() {
    test_112()
}
