module tests.test_865;

@Getter
class User_865 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_865() {
    let u = User_865(865, "User_865")
    assert(u.getId() == 865)
}


pub fn main() {
    test_865()
}
