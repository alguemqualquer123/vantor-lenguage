module tests.test_293;

@Getter
class User_293 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_293() {
    let u = User_293(293, "User_293")
    assert(u.getId() == 293)
}


pub fn main() {
    test_293()
}
