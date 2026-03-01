module tests.test_658;

@Getter
class User_658 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_658() {
    let u = User_658(658, "User_658")
    assert(u.getId() == 658)
}


pub fn main() {
    test_658()
}
