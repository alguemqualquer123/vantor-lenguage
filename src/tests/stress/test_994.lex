module tests.test_994;

@Getter
class User_994 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_994() {
    let u = User_994(994, "User_994")
    assert(u.getId() == 994)
}


pub fn main() {
    test_994()
}
