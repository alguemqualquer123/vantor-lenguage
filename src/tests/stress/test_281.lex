module tests.test_281;

@Getter
class User_281 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_281() {
    let u = User_281(281, "User_281")
    assert(u.getId() == 281)
}


pub fn main() {
    test_281()
}
