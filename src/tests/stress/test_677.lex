module tests.test_677;

@Getter
class User_677 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_677() {
    let u = User_677(677, "User_677")
    assert(u.getId() == 677)
}


pub fn main() {
    test_677()
}
