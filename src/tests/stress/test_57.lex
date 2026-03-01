module tests.test_57;

@Getter
class User_57 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_57() {
    let u = User_57(57, "User_57")
    assert(u.getId() == 57)
}


pub fn main() {
    test_57()
}
