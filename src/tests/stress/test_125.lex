module tests.test_125;

@Getter
class User_125 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_125() {
    let u = User_125(125, "User_125")
    assert(u.getId() == 125)
}


pub fn main() {
    test_125()
}
