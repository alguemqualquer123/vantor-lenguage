module tests.test_151;

@Getter
class User_151 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_151() {
    let u = User_151(151, "User_151")
    assert(u.getId() == 151)
}


pub fn main() {
    test_151()
}
