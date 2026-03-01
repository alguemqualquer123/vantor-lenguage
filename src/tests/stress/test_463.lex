module tests.test_463;

@Getter
class User_463 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_463() {
    let u = User_463(463, "User_463")
    assert(u.getId() == 463)
}


pub fn main() {
    test_463()
}
