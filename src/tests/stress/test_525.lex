module tests.test_525;

@Getter
class User_525 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_525() {
    let u = User_525(525, "User_525")
    assert(u.getId() == 525)
}


pub fn main() {
    test_525()
}
