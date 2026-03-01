module tests.test_893;

@Getter
class User_893 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_893() {
    let u = User_893(893, "User_893")
    assert(u.getId() == 893)
}


pub fn main() {
    test_893()
}
