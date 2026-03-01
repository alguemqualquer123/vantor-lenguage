module tests.test_82;

@Getter
class User_82 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_82() {
    let u = User_82(82, "User_82")
    assert(u.getId() == 82)
}


pub fn main() {
    test_82()
}
