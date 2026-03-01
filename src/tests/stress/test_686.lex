module tests.test_686;

@Getter
class User_686 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_686() {
    let u = User_686(686, "User_686")
    assert(u.getId() == 686)
}


pub fn main() {
    test_686()
}
