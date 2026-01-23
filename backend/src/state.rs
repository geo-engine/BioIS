use crate::{
    auth::{GeoEngineAuthMiddleware, User},
    db::DbPool,
    jobs::JobHandler,
    util::write_lock,
};
use ogcapi::{
    processes::Processor,
    services::{OgcApiProcessesState, OgcApiState},
    types::common::{Conformance, LandingPage, Link},
};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

pub type BoxedProcessor = Box<dyn Processor<User = User>>;

#[derive(Clone)]
pub struct AppState {
    pub root: Arc<RwLock<LandingPage>>,
    pub conformance: Arc<RwLock<Conformance>>,
    pub jobs: Arc<JobHandler>,
    pub processors: Arc<RwLock<HashMap<String, BoxedProcessor>>>,
}

impl AppState {
    pub fn new(database_pool: DbPool) -> Self {
        Self {
            root: Arc::new(RwLock::new(LandingPage::default())),
            conformance: Arc::new(RwLock::new(Conformance::default())),
            jobs: Arc::new(JobHandler {
                connection: database_pool,
            }),
            processors: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn with_processors(self, processors: impl IntoIterator<Item = BoxedProcessor>) -> Self {
        {
            let mut proc_map = write_lock(&self.processors);
            for processor in processors {
                proc_map.insert(processor.id().to_string(), processor);
            }
        }
        self
    }
}

impl OgcApiState for AppState {
    type User = User;
    type AuthLayer = GeoEngineAuthMiddleware;

    fn root(&self) -> LandingPage {
        use crate::util::read_lock;

        read_lock(self.root.as_ref()).to_owned()
    }

    fn add_links(&mut self, links: impl IntoIterator<Item = Link>) {
        use crate::util::write_lock;

        write_lock(&self.root).links.extend(links);
    }

    fn conformance(&self) -> Conformance {
        use crate::util::read_lock;

        read_lock(self.conformance.as_ref()).to_owned()
    }

    fn extend_conformance(&self, items: &[&str]) {
        use crate::util::write_lock;

        write_lock(&self.conformance).extend(items);
    }

    fn auth_middleware(&self) -> Self::AuthLayer {
        GeoEngineAuthMiddleware::new()
    }
}

impl OgcApiProcessesState for AppState {
    fn processors(&self) -> Vec<Box<dyn Processor<User = Self::User>>> {
        use crate::util::read_lock;

        read_lock(self.processors.as_ref())
            .values()
            .cloned()
            .collect()
    }

    fn processor(&self, id: &str) -> Option<Box<dyn Processor<User = Self::User>>> {
        use crate::util::read_lock;

        read_lock(self.processors.as_ref()).get(id).cloned()
    }

    fn jobs(&self) -> &dyn ogcapi::drivers::JobHandler<User = Self::User> {
        &*self.jobs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::CONFIG,
        db::{DbPool, setup_db},
        util::Secret,
    };
    use ogcapi::{processes::echo::Echo, types::common::Link};

    fn dummy_db_pool() -> DbPool {
        setup_db(&CONFIG.database).unwrap()
    }

    #[test]
    fn test_app_state_new_initializes_fields() {
        let db_pool = dummy_db_pool();
        let state = AppState::new(db_pool);

        assert_eq!(state.root.read().unwrap().links.len(), 0);
        assert_eq!(state.conformance.read().unwrap().conforms_to.len(), 0);
        assert!(state.processors.read().unwrap().is_empty());
    }

    #[test]
    fn test_with_processors_adds_processor() {
        let db_pool = dummy_db_pool();
        let state = AppState::new(db_pool);
        let processor: BoxedProcessor = Box::<Echo<_>>::default();

        let state = state.with_processors(vec![processor]);
        let processors = state.processors.read().unwrap();
        assert!(processors.contains_key("echo"));
    }

    #[test]
    fn test_add_links_extends_landing_page_links() {
        let db_pool = dummy_db_pool();
        let mut state = AppState::new(db_pool);

        let links = vec![Link::new("a", "self"), Link::new("b", "alternate")];
        state.add_links(links.clone());

        let root = state.root.read().unwrap();
        assert!(root.links.iter().any(|l| l.href == "a"));
        assert!(root.links.iter().any(|l| l.href == "b"));
    }

    #[test]
    fn test_extend_conformance_adds_items() {
        let db_pool = dummy_db_pool();
        let state = AppState::new(db_pool);

        state.extend_conformance(&["http://example.com/spec1", "http://example.com/spec2"]);
        let conformance = state.conformance.read().unwrap();
        assert!(
            conformance
                .conforms_to
                .contains(&"http://example.com/spec1".to_string())
        );
        assert!(
            conformance
                .conforms_to
                .contains(&"http://example.com/spec2".to_string())
        );
    }

    #[test]
    fn test_processor_and_processors_methods() {
        let db_pool = dummy_db_pool();
        let state = AppState::new(db_pool);
        let processor: BoxedProcessor = Box::<Echo<_>>::default();

        let state = state.with_processors(vec![processor]);
        let all = state.processors();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].id(), "echo");

        let found = state.processor("echo");
        assert!(found.is_some());
        assert_eq!(found.unwrap().id(), "echo");

        let not_found = state.processor("not_found");
        assert!(not_found.is_none());
    }

    #[tokio::test]
    async fn test_jobs_returns_job_handler() {
        let db_pool = dummy_db_pool();
        let state = AppState::new(db_pool);

        let jobs = state.jobs();

        jobs.status_list(
            0,
            1,
            &User {
                id: Default::default(),
                session_token: Secret(Default::default()),
            },
        )
        .await
        .unwrap();
    }
}
