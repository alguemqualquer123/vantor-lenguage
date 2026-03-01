module tests.test_180;

@Getter
class User_180 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_180() {
    let u = User_180(180, "User_180")
    assert(u.getId() == 180)
}


pub fn main() {
    test_180()
}
