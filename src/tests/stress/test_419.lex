module tests.test_419;

@Getter
class User_419 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_419() {
    let u = User_419(419, "User_419")
    assert(u.getId() == 419)
}


pub fn main() {
    test_419()
}
