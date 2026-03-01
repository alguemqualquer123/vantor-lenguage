module tests.test_310;

@Getter
class User_310 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_310() {
    let u = User_310(310, "User_310")
    assert(u.getId() == 310)
}


pub fn main() {
    test_310()
}
