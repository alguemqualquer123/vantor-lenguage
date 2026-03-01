module tests.test_581;

@Getter
class User_581 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_581() {
    let u = User_581(581, "User_581")
    assert(u.getId() == 581)
}


pub fn main() {
    test_581()
}
