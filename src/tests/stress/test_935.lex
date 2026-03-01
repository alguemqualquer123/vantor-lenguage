module tests.test_935;

@Getter
class User_935 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_935() {
    let u = User_935(935, "User_935")
    assert(u.getId() == 935)
}


pub fn main() {
    test_935()
}
