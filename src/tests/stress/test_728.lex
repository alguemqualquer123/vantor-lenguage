module tests.test_728;

@Getter
class User_728 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_728() {
    let u = User_728(728, "User_728")
    assert(u.getId() == 728)
}


pub fn main() {
    test_728()
}
