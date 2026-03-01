module tests.test_880;

@Getter
class User_880 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_880() {
    let u = User_880(880, "User_880")
    assert(u.getId() == 880)
}


pub fn main() {
    test_880()
}
