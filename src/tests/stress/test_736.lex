module tests.test_736;

@Getter
class User_736 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_736() {
    let u = User_736(736, "User_736")
    assert(u.getId() == 736)
}


pub fn main() {
    test_736()
}
