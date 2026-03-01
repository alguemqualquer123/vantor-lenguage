module tests.test_250;

@Getter
class User_250 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_250() {
    let u = User_250(250, "User_250")
    assert(u.getId() == 250)
}


pub fn main() {
    test_250()
}
