module tests.test_836;

@Getter
class User_836 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_836() {
    let u = User_836(836, "User_836")
    assert(u.getId() == 836)
}


pub fn main() {
    test_836()
}
