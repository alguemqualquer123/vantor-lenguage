module tests.test_540;

@Getter
class User_540 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_540() {
    let u = User_540(540, "User_540")
    assert(u.getId() == 540)
}


pub fn main() {
    test_540()
}
