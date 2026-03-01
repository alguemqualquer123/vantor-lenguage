module tests.test_613;

@Getter
class User_613 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_613() {
    let u = User_613(613, "User_613")
    assert(u.getId() == 613)
}


pub fn main() {
    test_613()
}
