module tests.test_269;

@Getter
class User_269 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_269() {
    let u = User_269(269, "User_269")
    assert(u.getId() == 269)
}


pub fn main() {
    test_269()
}
