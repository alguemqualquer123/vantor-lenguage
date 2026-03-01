module tests.test_286;

@Getter
class User_286 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_286() {
    let u = User_286(286, "User_286")
    assert(u.getId() == 286)
}


pub fn main() {
    test_286()
}
