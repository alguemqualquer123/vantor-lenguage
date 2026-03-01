module tests.test_625;

@Getter
class User_625 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_625() {
    let u = User_625(625, "User_625")
    assert(u.getId() == 625)
}


pub fn main() {
    test_625()
}
