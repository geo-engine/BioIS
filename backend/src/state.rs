use crate::auth::User;
use tokio::task_local;

task_local! {
   pub static USER: User;
}

pub fn spawn_with_user<F>(fut: F) -> tokio::task::JoinHandle<()>
where
    F: futures::Future<Output = ()> + Send + 'static,
{
    let Ok(user) = USER.try_get() else {
        return tokio::spawn(fut); // fallback if no user is set
    };

    tokio::spawn(USER.scope(user, fut))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_spawns_with_user() {
        let user = User {
            id: uuid::Uuid::from_u128(42),
            session_token: uuid::Uuid::from_u128(42).into(),
        };

        let (tx, rx) = tokio::sync::oneshot::channel();

        // set user in this scope
        USER.scope(user.clone(), async {
            spawn_with_user(async {
                let current_user = USER.get();
                tx.send(current_user.id).unwrap();
            })
            .await
            .unwrap();
        })
        .await;

        tokio::select! {
            () = tokio::time::sleep(std::time::Duration::from_secs(1)) => {
                panic!("Timeout waiting for result");
            }
            foo = rx => {
                assert_eq!(foo.unwrap(), user.id);
            }
        }
    }
}
