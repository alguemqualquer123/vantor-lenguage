module tests.test_50;

@Getter
class User_50 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_50() {
    let u = User_50(50, "User_50")
    assert(u.getId() == 50)
}


pub fn main() {
    test_50()
}
