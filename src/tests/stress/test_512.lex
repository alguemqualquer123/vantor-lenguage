module tests.test_512;

@Getter
class User_512 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_512() {
    let u = User_512(512, "User_512")
    assert(u.getId() == 512)
}


pub fn main() {
    test_512()
}
