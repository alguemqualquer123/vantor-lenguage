module tests.test_498;

@Getter
class User_498 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_498() {
    let u = User_498(498, "User_498")
    assert(u.getId() == 498)
}


pub fn main() {
    test_498()
}
