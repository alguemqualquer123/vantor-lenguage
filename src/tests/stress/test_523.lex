module tests.test_523;

@Getter
class User_523 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_523() {
    let u = User_523(523, "User_523")
    assert(u.getId() == 523)
}


pub fn main() {
    test_523()
}
