use std::{sync::Arc, time::Duration, ops::Deref};
use mutflex::Mutex;
use futures::future::{join_all, try_join_all};
use rand::random;

#[test]
fn sync () {
    let mutex = Arc::new(Mutex::new(1));
    let m2 = mutex.clone();

    let join = std::thread::spawn(move || {
        let mut mutex = m2.lock();
        *mutex += 1;
    });

    let mut lock = mutex.lock();
    *lock += 1;
    drop(lock);

    join.join().unwrap();
    let lock = mutex.lock();
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
        let mut mutex = m2.lock();
        *mutex += 1;
    });

    let mut lock = mutex.lock_async().await;
    *lock += 1;
    drop(lock);

    join.join().unwrap();
    let lock = mutex.lock();
    let inner = *lock;
    assert_eq!(inner, 3);
}

#[tokio::test(flavor = "multi_thread")]
async fn stress () {
    let epochs = 5000;
    let mutex = Arc::new(Mutex::new(0));
    let mut handles = Vec::with_capacity(epochs);

    for _ in 0..epochs {
        let mutex = mutex.clone();
        let handle = tokio::spawn(async move {
            let mut mutex = mutex.lock_async().await;
            tokio::time::sleep(Duration::from_secs_f32(0.01f32 * random::<f32>())).await;
            *mutex += 1;
        });

        handles.push(handle);
    }

    try_join_all(handles).await.unwrap();
    let result = mutex.try_into_inner_arc().unwrap();
    assert_eq!(epochs, result)
}