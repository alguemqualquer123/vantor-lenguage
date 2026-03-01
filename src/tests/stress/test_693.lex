module tests.test_693;

@Getter
class User_693 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_693() {
    let u = User_693(693, "User_693")
    assert(u.getId() == 693)
}


pub fn main() {
    test_693()
}
