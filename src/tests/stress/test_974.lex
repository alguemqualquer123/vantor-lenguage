module tests.test_974;

@Getter
class User_974 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_974() {
    let u = User_974(974, "User_974")
    assert(u.getId() == 974)
}


pub fn main() {
    test_974()
}
