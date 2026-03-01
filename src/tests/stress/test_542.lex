module tests.test_542;

@Getter
class User_542 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_542() {
    let u = User_542(542, "User_542")
    assert(u.getId() == 542)
}


pub fn main() {
    test_542()
}
