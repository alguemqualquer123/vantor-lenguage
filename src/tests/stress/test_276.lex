module tests.test_276;

@Getter
class User_276 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_276() {
    let u = User_276(276, "User_276")
    assert(u.getId() == 276)
}


pub fn main() {
    test_276()
}
