module tests.test_902;

@Getter
class User_902 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_902() {
    let u = User_902(902, "User_902")
    assert(u.getId() == 902)
}


pub fn main() {
    test_902()
}
