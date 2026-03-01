module tests.test_712;

@Getter
class User_712 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_712() {
    let u = User_712(712, "User_712")
    assert(u.getId() == 712)
}


pub fn main() {
    test_712()
}
