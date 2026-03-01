module tests.test_950;

@Getter
class User_950 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_950() {
    let u = User_950(950, "User_950")
    assert(u.getId() == 950)
}


pub fn main() {
    test_950()
}
