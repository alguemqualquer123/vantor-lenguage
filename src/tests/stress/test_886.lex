module tests.test_886;

@Getter
class User_886 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_886() {
    let u = User_886(886, "User_886")
    assert(u.getId() == 886)
}


pub fn main() {
    test_886()
}
