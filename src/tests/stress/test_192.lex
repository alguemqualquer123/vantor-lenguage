module tests.test_192;

@Getter
class User_192 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_192() {
    let u = User_192(192, "User_192")
    assert(u.getId() == 192)
}


pub fn main() {
    test_192()
}
