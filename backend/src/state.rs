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
