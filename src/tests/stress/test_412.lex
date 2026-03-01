module tests.test_412;

@Getter
class User_412 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_412() {
    let u = User_412(412, "User_412")
    assert(u.getId() == 412)
}


pub fn main() {
    test_412()
}
