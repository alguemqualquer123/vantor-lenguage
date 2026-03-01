module tests.test_806;

@Getter
class User_806 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_806() {
    let u = User_806(806, "User_806")
    assert(u.getId() == 806)
}


pub fn main() {
    test_806()
}
