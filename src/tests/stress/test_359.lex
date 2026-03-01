module tests.test_359;

@Getter
class User_359 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_359() {
    let u = User_359(359, "User_359")
    assert(u.getId() == 359)
}


pub fn main() {
    test_359()
}
