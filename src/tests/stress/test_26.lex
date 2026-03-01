module tests.test_26;

@Getter
class User_26 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_26() {
    let u = User_26(26, "User_26")
    assert(u.getId() == 26)
}


pub fn main() {
    test_26()
}
