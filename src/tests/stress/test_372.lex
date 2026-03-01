module tests.test_372;

@Getter
class User_372 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_372() {
    let u = User_372(372, "User_372")
    assert(u.getId() == 372)
}


pub fn main() {
    test_372()
}
