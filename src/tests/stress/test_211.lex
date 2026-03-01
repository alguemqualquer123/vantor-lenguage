module tests.test_211;

@Getter
class User_211 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_211() {
    let u = User_211(211, "User_211")
    assert(u.getId() == 211)
}


pub fn main() {
    test_211()
}
