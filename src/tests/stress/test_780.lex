module tests.test_780;

@Getter
class User_780 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_780() {
    let u = User_780(780, "User_780")
    assert(u.getId() == 780)
}


pub fn main() {
    test_780()
}
