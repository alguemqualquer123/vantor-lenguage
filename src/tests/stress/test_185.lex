module tests.test_185;

@Getter
class User_185 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_185() {
    let u = User_185(185, "User_185")
    assert(u.getId() == 185)
}


pub fn main() {
    test_185()
}
