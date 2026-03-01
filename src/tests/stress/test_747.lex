module tests.test_747;

@Getter
class User_747 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_747() {
    let u = User_747(747, "User_747")
    assert(u.getId() == 747)
}


pub fn main() {
    test_747()
}
