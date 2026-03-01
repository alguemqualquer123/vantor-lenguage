module tests.test_278;

@Getter
class User_278 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_278() {
    let u = User_278(278, "User_278")
    assert(u.getId() == 278)
}


pub fn main() {
    test_278()
}
