module tests.test_582;

@Getter
class User_582 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_582() {
    let u = User_582(582, "User_582")
    assert(u.getId() == 582)
}


pub fn main() {
    test_582()
}
