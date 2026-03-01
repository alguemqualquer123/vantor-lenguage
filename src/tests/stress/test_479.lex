module tests.test_479;

@Getter
class User_479 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_479() {
    let u = User_479(479, "User_479")
    assert(u.getId() == 479)
}


pub fn main() {
    test_479()
}
