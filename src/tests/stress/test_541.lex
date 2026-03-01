module tests.test_541;

@Getter
class User_541 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_541() {
    let u = User_541(541, "User_541")
    assert(u.getId() == 541)
}


pub fn main() {
    test_541()
}
