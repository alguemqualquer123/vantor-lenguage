module tests.test_556;

@Getter
class User_556 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_556() {
    let u = User_556(556, "User_556")
    assert(u.getId() == 556)
}


pub fn main() {
    test_556()
}
