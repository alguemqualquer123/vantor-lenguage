module tests.test_953;

@Getter
class User_953 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_953() {
    let u = User_953(953, "User_953")
    assert(u.getId() == 953)
}


pub fn main() {
    test_953()
}
