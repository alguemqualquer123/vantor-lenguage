module tests.test_933;

@Getter
class User_933 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_933() {
    let u = User_933(933, "User_933")
    assert(u.getId() == 933)
}


pub fn main() {
    test_933()
}
