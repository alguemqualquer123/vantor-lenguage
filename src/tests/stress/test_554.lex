module tests.test_554;

@Getter
class User_554 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_554() {
    let u = User_554(554, "User_554")
    assert(u.getId() == 554)
}


pub fn main() {
    test_554()
}
