module tests.test_474;

@Getter
class User_474 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_474() {
    let u = User_474(474, "User_474")
    assert(u.getId() == 474)
}


pub fn main() {
    test_474()
}
