use std::sync::Arc;
use mutflex::Mutex;

#[test]
fn sync () {
    let mutex = Arc::new(Mutex::new(1));
    let m2 = mutex.clone();

    let join = std::thread::spawn(move || {
        let mut mutex = m2.lock_block();
        *mutex += 1;
    });

    let mut lock = mutex.lock_block();
    *lock += 1;
    drop(lock);

    join.join().unwrap();
    let lock = mutex.lock_block();
    let inner = *lock;
    assert_eq!(inner, 3);
}

#[tokio::test]
async fn r#async () {
    let mutex = Arc::new(Mutex::new(1));
    let m2 = mutex.clone();

    let join = tokio::spawn(async move {
        let mut mutex = m2.lock_async().await;
        *mutex += 1;
    });

    let mut lock = mutex.lock_async().await;
    *lock += 1;
    drop(lock);

    join.await.unwrap();
    let lock = mutex.lock_async().await;
    let inner = *lock;
    assert_eq!(inner, 3);
}

#[tokio::test]
async fn mixed () {
    let mutex = Arc::new(Mutex::new(1));
    let m2 = mutex.clone();

    let join = std::thread::spawn(move || {
        let mut mutex = m2.lock_block();
        *mutex += 1;
    });

    let mut lock = mutex.lock_async().await;
    *lock += 1;
    drop(lock);

    join.join().unwrap();
    let lock = mutex.lock_block();
    let inner = *lock;
    assert_eq!(inner, 3);
}