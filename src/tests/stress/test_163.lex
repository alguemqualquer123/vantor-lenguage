module tests.test_163;

@Getter
class User_163 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_163() {
    let u = User_163(163, "User_163")
    assert(u.getId() == 163)
}


pub fn main() {
    test_163()
}
