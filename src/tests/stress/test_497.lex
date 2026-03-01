module tests.test_497;

@Getter
class User_497 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_497() {
    let u = User_497(497, "User_497")
    assert(u.getId() == 497)
}


pub fn main() {
    test_497()
}
