module tests.test_833;

@Getter
class User_833 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_833() {
    let u = User_833(833, "User_833")
    assert(u.getId() == 833)
}


pub fn main() {
    test_833()
}
