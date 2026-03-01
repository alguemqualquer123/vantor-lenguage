module tests.test_244;

@Getter
class User_244 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_244() {
    let u = User_244(244, "User_244")
    assert(u.getId() == 244)
}


pub fn main() {
    test_244()
}
