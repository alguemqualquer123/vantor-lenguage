module tests.test_158;

@Getter
class User_158 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_158() {
    let u = User_158(158, "User_158")
    assert(u.getId() == 158)
}


pub fn main() {
    test_158()
}
