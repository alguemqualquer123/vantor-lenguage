module tests.test_856;

@Getter
class User_856 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_856() {
    let u = User_856(856, "User_856")
    assert(u.getId() == 856)
}


pub fn main() {
    test_856()
}
