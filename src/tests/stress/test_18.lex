module tests.test_18;

@Getter
class User_18 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_18() {
    let u = User_18(18, "User_18")
    assert(u.getId() == 18)
}


pub fn main() {
    test_18()
}
