module tests.test_194;

@Getter
class User_194 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_194() {
    let u = User_194(194, "User_194")
    assert(u.getId() == 194)
}


pub fn main() {
    test_194()
}
