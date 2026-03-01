module tests.test_349;

@Getter
class User_349 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_349() {
    let u = User_349(349, "User_349")
    assert(u.getId() == 349)
}


pub fn main() {
    test_349()
}
