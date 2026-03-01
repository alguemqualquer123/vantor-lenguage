module tests.test_150;

@Getter
class User_150 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_150() {
    let u = User_150(150, "User_150")
    assert(u.getId() == 150)
}


pub fn main() {
    test_150()
}
