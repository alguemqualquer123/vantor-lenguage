module tests.test_398;

@Getter
class User_398 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_398() {
    let u = User_398(398, "User_398")
    assert(u.getId() == 398)
}


pub fn main() {
    test_398()
}
