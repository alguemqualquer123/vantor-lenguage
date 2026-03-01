module tests.test_522;

@Getter
class User_522 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_522() {
    let u = User_522(522, "User_522")
    assert(u.getId() == 522)
}


pub fn main() {
    test_522()
}
