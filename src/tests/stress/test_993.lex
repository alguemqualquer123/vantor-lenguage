module tests.test_993;

@Getter
class User_993 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_993() {
    let u = User_993(993, "User_993")
    assert(u.getId() == 993)
}


pub fn main() {
    test_993()
}
