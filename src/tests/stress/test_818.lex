module tests.test_818;

@Getter
class User_818 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_818() {
    let u = User_818(818, "User_818")
    assert(u.getId() == 818)
}


pub fn main() {
    test_818()
}
