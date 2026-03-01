module tests.test_788;

@Getter
class User_788 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_788() {
    let u = User_788(788, "User_788")
    assert(u.getId() == 788)
}


pub fn main() {
    test_788()
}
