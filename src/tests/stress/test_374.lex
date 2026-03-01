module tests.test_374;

@Getter
class User_374 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_374() {
    let u = User_374(374, "User_374")
    assert(u.getId() == 374)
}


pub fn main() {
    test_374()
}
