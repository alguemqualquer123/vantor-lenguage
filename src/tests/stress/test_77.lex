module tests.test_77;

@Getter
class User_77 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_77() {
    let u = User_77(77, "User_77")
    assert(u.getId() == 77)
}


pub fn main() {
    test_77()
}
