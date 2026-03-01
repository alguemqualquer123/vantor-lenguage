module tests.test_714;

@Getter
class User_714 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_714() {
    let u = User_714(714, "User_714")
    assert(u.getId() == 714)
}


pub fn main() {
    test_714()
}
