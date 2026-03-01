module tests.test_261;

@Getter
class User_261 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_261() {
    let u = User_261(261, "User_261")
    assert(u.getId() == 261)
}


pub fn main() {
    test_261()
}
