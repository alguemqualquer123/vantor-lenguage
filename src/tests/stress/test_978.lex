module tests.test_978;

@Getter
class User_978 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_978() {
    let u = User_978(978, "User_978")
    assert(u.getId() == 978)
}


pub fn main() {
    test_978()
}
