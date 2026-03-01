module tests.test_93;

@Getter
class User_93 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_93() {
    let u = User_93(93, "User_93")
    assert(u.getId() == 93)
}


pub fn main() {
    test_93()
}
