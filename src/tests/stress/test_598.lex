module tests.test_598;

@Getter
class User_598 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_598() {
    let u = User_598(598, "User_598")
    assert(u.getId() == 598)
}


pub fn main() {
    test_598()
}
