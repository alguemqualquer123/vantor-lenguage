module tests.test_441;

@Getter
class User_441 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_441() {
    let u = User_441(441, "User_441")
    assert(u.getId() == 441)
}


pub fn main() {
    test_441()
}
