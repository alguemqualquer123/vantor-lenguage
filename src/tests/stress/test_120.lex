module tests.test_120;

@Getter
class User_120 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_120() {
    let u = User_120(120, "User_120")
    assert(u.getId() == 120)
}


pub fn main() {
    test_120()
}
