module tests.test_507;

@Getter
class User_507 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_507() {
    let u = User_507(507, "User_507")
    assert(u.getId() == 507)
}


pub fn main() {
    test_507()
}
