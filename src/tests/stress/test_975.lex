module tests.test_975;

@Getter
class User_975 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_975() {
    let u = User_975(975, "User_975")
    assert(u.getId() == 975)
}


pub fn main() {
    test_975()
}
