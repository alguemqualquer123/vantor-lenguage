module tests.test_141;

@Getter
class User_141 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_141() {
    let u = User_141(141, "User_141")
    assert(u.getId() == 141)
}


pub fn main() {
    test_141()
}
