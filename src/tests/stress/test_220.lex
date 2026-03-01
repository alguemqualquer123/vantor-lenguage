module tests.test_220;

@Getter
class User_220 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_220() {
    let u = User_220(220, "User_220")
    assert(u.getId() == 220)
}


pub fn main() {
    test_220()
}
