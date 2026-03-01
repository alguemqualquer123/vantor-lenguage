module tests.test_464;

@Getter
class User_464 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_464() {
    let u = User_464(464, "User_464")
    assert(u.getId() == 464)
}


pub fn main() {
    test_464()
}
