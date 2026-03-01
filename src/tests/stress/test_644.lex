module tests.test_644;

@Getter
class User_644 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_644() {
    let u = User_644(644, "User_644")
    assert(u.getId() == 644)
}


pub fn main() {
    test_644()
}
