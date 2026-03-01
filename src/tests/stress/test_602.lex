module tests.test_602;

@Getter
class User_602 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_602() {
    let u = User_602(602, "User_602")
    assert(u.getId() == 602)
}


pub fn main() {
    test_602()
}
