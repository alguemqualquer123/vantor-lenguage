module tests.test_181;

@Getter
class User_181 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_181() {
    let u = User_181(181, "User_181")
    assert(u.getId() == 181)
}


pub fn main() {
    test_181()
}
