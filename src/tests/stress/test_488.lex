module tests.test_488;

@Getter
class User_488 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_488() {
    let u = User_488(488, "User_488")
    assert(u.getId() == 488)
}


pub fn main() {
    test_488()
}
