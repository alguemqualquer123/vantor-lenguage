module tests.test_459;

@Getter
class User_459 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_459() {
    let u = User_459(459, "User_459")
    assert(u.getId() == 459)
}


pub fn main() {
    test_459()
}
