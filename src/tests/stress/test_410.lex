module tests.test_410;

@Getter
class User_410 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_410() {
    let u = User_410(410, "User_410")
    assert(u.getId() == 410)
}


pub fn main() {
    test_410()
}
