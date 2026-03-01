module tests.test_870;

@Getter
class User_870 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_870() {
    let u = User_870(870, "User_870")
    assert(u.getId() == 870)
}


pub fn main() {
    test_870()
}
