module tests.test_854;

@Getter
class User_854 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_854() {
    let u = User_854(854, "User_854")
    assert(u.getId() == 854)
}


pub fn main() {
    test_854()
}
