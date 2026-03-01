module tests.test_122;

@Getter
class User_122 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_122() {
    let u = User_122(122, "User_122")
    assert(u.getId() == 122)
}


pub fn main() {
    test_122()
}
