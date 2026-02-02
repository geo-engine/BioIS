use ogcapi::{
    drivers::CollectionTransactions,
    types::common::{Collection, Collections, Query as CollectionQuery},
};

pub struct NoCollectionTransactions;

#[async_trait::async_trait]
impl CollectionTransactions for NoCollectionTransactions {
    async fn create_collection(&self, _collection: &Collection) -> anyhow::Result<String> {
        Err(anyhow::anyhow!("Collection transactions are not supported"))
    }

    async fn read_collection(&self, _id: &str) -> anyhow::Result<Option<Collection>> {
        Err(anyhow::anyhow!("Collection transactions are not supported"))
    }

    async fn update_collection(&self, _collection: &Collection) -> anyhow::Result<()> {
        Err(anyhow::anyhow!("Collection transactions are not supported"))
    }

    async fn delete_collection(&self, _id: &str) -> anyhow::Result<()> {
        Err(anyhow::anyhow!("Collection transactions are not supported"))
    }

    async fn list_collections(&self, _query: &CollectionQuery) -> anyhow::Result<Collections> {
        Err(anyhow::anyhow!("Collection transactions are not supported"))
    }
}
