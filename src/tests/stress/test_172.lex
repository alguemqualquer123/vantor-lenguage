module tests.test_172;

@Getter
class User_172 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_172() {
    let u = User_172(172, "User_172")
    assert(u.getId() == 172)
}


pub fn main() {
    test_172()
}
