module tests.test_962;

@Getter
class User_962 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_962() {
    let u = User_962(962, "User_962")
    assert(u.getId() == 962)
}


pub fn main() {
    test_962()
}
