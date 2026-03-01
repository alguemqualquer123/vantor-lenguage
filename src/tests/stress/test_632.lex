module tests.test_632;

@Getter
class User_632 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_632() {
    let u = User_632(632, "User_632")
    assert(u.getId() == 632)
}


pub fn main() {
    test_632()
}
