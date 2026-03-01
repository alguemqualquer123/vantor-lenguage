module tests.test_388;

@Getter
class User_388 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_388() {
    let u = User_388(388, "User_388")
    assert(u.getId() == 388)
}


pub fn main() {
    test_388()
}
