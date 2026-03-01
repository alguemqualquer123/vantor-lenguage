module tests.test_87;

@Getter
class User_87 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_87() {
    let u = User_87(87, "User_87")
    assert(u.getId() == 87)
}


pub fn main() {
    test_87()
}
