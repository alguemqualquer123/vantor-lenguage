use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;

/// Representa o container de Injeção de Dependência global.
pub struct DependencyContainer {
    beans: Mutex<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>,
}

impl DependencyContainer {
    pub fn new() -> Self {
        Self {
            beans: Mutex::new(HashMap::new()),
        }
    }

    /// Registra uma dependência no container.
    pub fn register<T: 'static + Send + Sync>(&self, component: T) {
        let mut beans = self.beans.lock().unwrap();
        beans.insert(TypeId::of::<T>(), Arc::new(component));
    }

    /// Resolve uma dependência do container.
    pub fn resolve<T: 'static + Send + Sync>(&self) -> Option<Arc<T>> {
        let beans = self.beans.lock().unwrap();
        beans.get(&TypeId::of::<T>())
            .and_then(|any| any.clone().downcast::<T>().ok())
    }
}

/// Instância global do container de DI para uso no runtime.
pub static CONTAINER: Lazy<DependencyContainer> = Lazy::new(|| DependencyContainer::new());

/// Macro simulada para injeção de dependência no Lexicon.
/// No compilador real, isso seria traduzido para chamadas ao CONTAINER.
pub fn inject<T: 'static + Send + Sync>() -> Arc<T> {
    CONTAINER.resolve::<T>().expect("Dependência não encontrada no container!")
}
