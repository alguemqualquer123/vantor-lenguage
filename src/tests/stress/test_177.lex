module tests.test_177;

@Getter
class User_177 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_177() {
    let u = User_177(177, "User_177")
    assert(u.getId() == 177)
}


pub fn main() {
    test_177()
}
