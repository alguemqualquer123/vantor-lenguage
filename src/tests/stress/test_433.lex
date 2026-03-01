module tests.test_433;

@Getter
class User_433 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_433() {
    let u = User_433(433, "User_433")
    assert(u.getId() == 433)
}


pub fn main() {
    test_433()
}
