module tests.test_616;

@Getter
class User_616 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_616() {
    let u = User_616(616, "User_616")
    assert(u.getId() == 616)
}


pub fn main() {
    test_616()
}
