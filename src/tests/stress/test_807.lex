module tests.test_807;

@Getter
class User_807 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_807() {
    let u = User_807(807, "User_807")
    assert(u.getId() == 807)
}


pub fn main() {
    test_807()
}
