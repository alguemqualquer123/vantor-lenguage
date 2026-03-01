module tests.test_436;

@Getter
class User_436 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_436() {
    let u = User_436(436, "User_436")
    assert(u.getId() == 436)
}


pub fn main() {
    test_436()
}
