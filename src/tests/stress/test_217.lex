module tests.test_217;

@Getter
class User_217 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_217() {
    let u = User_217(217, "User_217")
    assert(u.getId() == 217)
}


pub fn main() {
    test_217()
}
