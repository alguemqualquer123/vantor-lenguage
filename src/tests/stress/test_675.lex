module tests.test_675;

@Getter
class User_675 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_675() {
    let u = User_675(675, "User_675")
    assert(u.getId() == 675)
}


pub fn main() {
    test_675()
}
