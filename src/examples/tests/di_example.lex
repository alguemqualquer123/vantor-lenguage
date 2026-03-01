@Configuration
class AppConfig {
    @Bean
    fn userService() -> UserService {
        return UserService()
    }

    @Bean
    fn database() -> Database {
        return Database("localhost:5432")
    }
}

@Getter
@Setter
class User {
    @Inject
    pub id: i32;
    
    pub name: String;
    pub email: String;

    pub constructor(name: String, email: String) {
        self.name = name;
        self.email = email;
    }
}

class UserService {
    @Inject
    pub db: Database;

    @Test
    fn test_user_creation() {
        let user = User("Alice", "alice@lexicon.dev")
        assert(user.getName() == "Alice")
        user.setName("Bob")
        assert(user.getName() == "Bob")
    }
}

pub fn main() {
    // O sistema de DI do Lexicon processa as anotações @Configuration e @Bean
    let ctx = Lexicon::context(AppConfig)
    let service = ctx.get<UserService>()
    
    service.test_user_creation()
    println("DI and Decorators test passed!")
}
