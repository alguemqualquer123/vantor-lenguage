module tests.test_14;

@Getter
class User_14 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_14() {
    let u = User_14(14, "User_14")
    assert(u.getId() == 14)
}


pub fn main() {
    test_14()
}
