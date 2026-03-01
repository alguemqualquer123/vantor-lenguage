module tests.test_411;

@Getter
class User_411 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_411() {
    let u = User_411(411, "User_411")
    assert(u.getId() == 411)
}


pub fn main() {
    test_411()
}
