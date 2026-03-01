module tests.test_795;

@Getter
class User_795 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_795() {
    let u = User_795(795, "User_795")
    assert(u.getId() == 795)
}


pub fn main() {
    test_795()
}
