module tests.test_58;

@Getter
class User_58 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_58() {
    let u = User_58(58, "User_58")
    assert(u.getId() == 58)
}


pub fn main() {
    test_58()
}
