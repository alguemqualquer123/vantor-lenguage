module tests.test_544;

@Getter
class User_544 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_544() {
    let u = User_544(544, "User_544")
    assert(u.getId() == 544)
}


pub fn main() {
    test_544()
}
