module tests.test_306;

@Getter
class User_306 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_306() {
    let u = User_306(306, "User_306")
    assert(u.getId() == 306)
}


pub fn main() {
    test_306()
}
