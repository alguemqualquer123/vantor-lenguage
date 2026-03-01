module tests.test_951;

@Getter
class User_951 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_951() {
    let u = User_951(951, "User_951")
    assert(u.getId() == 951)
}


pub fn main() {
    test_951()
}
