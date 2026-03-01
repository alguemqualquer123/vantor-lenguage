module tests.test_882;

@Getter
class User_882 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_882() {
    let u = User_882(882, "User_882")
    assert(u.getId() == 882)
}


pub fn main() {
    test_882()
}
