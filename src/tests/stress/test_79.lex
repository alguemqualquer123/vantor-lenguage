module tests.test_79;

@Getter
class User_79 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_79() {
    let u = User_79(79, "User_79")
    assert(u.getId() == 79)
}


pub fn main() {
    test_79()
}
