use std::future::Future;
use std::time::Duration;

pub fn spawn<F>(_future: F) 
where 
    F: Future + Send + 'static,
{
}

pub async fn sleep(duration: Duration) {
    tokio::time::sleep(duration).await;
}

pub fn block_on<F>(future: F) -> F::Output 
where 
    F: Future,
{
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(future)
}
