module tests.test_876;

@Getter
class User_876 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_876() {
    let u = User_876(876, "User_876")
    assert(u.getId() == 876)
}


pub fn main() {
    test_876()
}
