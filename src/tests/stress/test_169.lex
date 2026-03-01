module tests.test_169;

@Getter
class User_169 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_169() {
    let u = User_169(169, "User_169")
    assert(u.getId() == 169)
}


pub fn main() {
    test_169()
}
