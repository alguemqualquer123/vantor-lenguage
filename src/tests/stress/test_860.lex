module tests.test_860;

@Getter
class User_860 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_860() {
    let u = User_860(860, "User_860")
    assert(u.getId() == 860)
}


pub fn main() {
    test_860()
}
