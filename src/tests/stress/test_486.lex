module tests.test_486;

@Getter
class User_486 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_486() {
    let u = User_486(486, "User_486")
    assert(u.getId() == 486)
}


pub fn main() {
    test_486()
}
