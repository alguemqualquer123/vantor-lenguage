module tests.test_145;

@Getter
class User_145 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_145() {
    let u = User_145(145, "User_145")
    assert(u.getId() == 145)
}


pub fn main() {
    test_145()
}
