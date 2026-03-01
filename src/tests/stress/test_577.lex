module tests.test_577;

@Getter
class User_577 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_577() {
    let u = User_577(577, "User_577")
    assert(u.getId() == 577)
}


pub fn main() {
    test_577()
}
