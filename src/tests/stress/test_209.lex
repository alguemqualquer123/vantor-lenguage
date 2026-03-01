module tests.test_209;

@Getter
class User_209 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_209() {
    let u = User_209(209, "User_209")
    assert(u.getId() == 209)
}


pub fn main() {
    test_209()
}
