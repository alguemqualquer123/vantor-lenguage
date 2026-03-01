module tests.test_591;

@Getter
class User_591 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_591() {
    let u = User_591(591, "User_591")
    assert(u.getId() == 591)
}


pub fn main() {
    test_591()
}
