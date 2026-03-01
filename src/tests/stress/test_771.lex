module tests.test_771;

@Getter
class User_771 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_771() {
    let u = User_771(771, "User_771")
    assert(u.getId() == 771)
}


pub fn main() {
    test_771()
}
