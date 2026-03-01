module tests.test_12;

@Getter
class User_12 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_12() {
    let u = User_12(12, "User_12")
    assert(u.getId() == 12)
}


pub fn main() {
    test_12()
}
