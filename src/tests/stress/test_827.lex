module tests.test_827;

@Getter
class User_827 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_827() {
    let u = User_827(827, "User_827")
    assert(u.getId() == 827)
}


pub fn main() {
    test_827()
}
