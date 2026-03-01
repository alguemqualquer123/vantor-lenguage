module tests.test_550;

@Getter
class User_550 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_550() {
    let u = User_550(550, "User_550")
    assert(u.getId() == 550)
}


pub fn main() {
    test_550()
}
