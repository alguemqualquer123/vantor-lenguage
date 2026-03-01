module tests.test_872;

@Getter
class User_872 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_872() {
    let u = User_872(872, "User_872")
    assert(u.getId() == 872)
}


pub fn main() {
    test_872()
}
