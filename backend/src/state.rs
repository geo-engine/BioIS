use crate::{
    auth::User,
    util::{Secret, read_lock, write_lock},
};
use anyhow::Result;
use std::sync::{Arc, RwLock};
use tokio::{task::LocalKey, task_local};
use tracing::Instrument;

task_local! {
   pub static CONTEXT: TaskContext;
}

#[derive(Clone, Debug)]
pub struct TaskContext {
    user: User,
    job_id: Arc<RwLock<Option<uuid::Uuid>>>,
}

impl TaskContext {
    pub fn new(user: User) -> Self {
        Self {
            user,
            job_id: Arc::new(RwLock::new(None)),
        }
    }

    pub fn set_job_id(&self, job_id: uuid::Uuid) {
        *write_lock(&self.job_id) = Some(job_id);
    }

    pub fn job_id(&self) -> Option<uuid::Uuid> {
        *read_lock(&self.job_id)
    }
}

pub trait TaskLocalContext {
    fn user_id(&'static self) -> Result<uuid::Uuid>;
    fn session_token(&'static self) -> Result<Secret<uuid::Uuid>>;
    fn job_id(&'static self) -> Result<Option<uuid::Uuid>>;
    fn set_job_id(&'static self, job_id: uuid::Uuid) -> Result<()>;
}

impl TaskLocalContext for LocalKey<TaskContext> {
    fn user_id(&'static self) -> Result<uuid::Uuid> {
        let context = self.try_get()?;
        Ok(context.user.id)
    }

    fn session_token(&'static self) -> Result<Secret<uuid::Uuid>> {
        let context = self.try_get()?;
        Ok(context.user.session_token)
    }

    fn job_id(&'static self) -> Result<Option<uuid::Uuid>> {
        let context = self.try_get()?;
        Ok(context.job_id())
    }

    fn set_job_id(&'static self, job_id: uuid::Uuid) -> Result<()> {
        let context = self.try_get()?;
        context.set_job_id(job_id);
        Ok(())
    }
}

pub fn spawn_with_user<F>(fut: F) -> tokio::task::JoinHandle<()>
where
    F: futures::Future<Output = ()> + Send + 'static,
{
    let Ok(context) = CONTEXT.try_get() else {
        return tokio::spawn(fut); // fallback if no user is set
    };

    tokio::spawn(CONTEXT.scope(context, fut.in_current_span()))
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
        CONTEXT
            .scope(TaskContext::new(user.clone()), async {
                spawn_with_user(async {
                    let current_user = CONTEXT.get();
                    tx.send(current_user.user.id).unwrap();
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
