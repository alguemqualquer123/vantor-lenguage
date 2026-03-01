module tests.test_434;

@Getter
class User_434 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_434() {
    let u = User_434(434, "User_434")
    assert(u.getId() == 434)
}


pub fn main() {
    test_434()
}
