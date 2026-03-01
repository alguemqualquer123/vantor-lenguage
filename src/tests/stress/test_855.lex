module tests.test_855;

@Getter
class User_855 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_855() {
    let u = User_855(855, "User_855")
    assert(u.getId() == 855)
}


pub fn main() {
    test_855()
}
