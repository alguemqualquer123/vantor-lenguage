module tests.test_770;

@Getter
class User_770 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_770() {
    let u = User_770(770, "User_770")
    assert(u.getId() == 770)
}


pub fn main() {
    test_770()
}
