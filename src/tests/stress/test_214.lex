module tests.test_214;

@Getter
class User_214 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_214() {
    let u = User_214(214, "User_214")
    assert(u.getId() == 214)
}


pub fn main() {
    test_214()
}
