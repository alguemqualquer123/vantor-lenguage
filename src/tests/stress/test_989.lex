module tests.test_989;

@Getter
class User_989 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_989() {
    let u = User_989(989, "User_989")
    assert(u.getId() == 989)
}


pub fn main() {
    test_989()
}
