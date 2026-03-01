module tests.test_769;

@Getter
class User_769 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_769() {
    let u = User_769(769, "User_769")
    assert(u.getId() == 769)
}


pub fn main() {
    test_769()
}
