use std::time::Instant;

use bson::{doc, oid::ObjectId};
use futures::StreamExt;
use log::info;
use mongodb::{
    options::{ClientOptions, FindOptions, ResolverConfig, UpdateModifications, UpdateOptions},
    results::{DeleteResult, UpdateResult},
    Client, Collection,
};

use crate::{
    config,
    model::{rule::Rule, token_data::TokenData},
};

#[derive(Debug)]
pub enum Error {
    BsonSer(bson::ser::Error),
    BsonOid(bson::oid::Error),
    MongoDb(mongodb::error::Error),
}

impl From<bson::ser::Error> for Error {
    fn from(e: bson::ser::Error) -> Self {
        Error::BsonSer(e)
    }
}

impl From<bson::oid::Error> for Error {
    fn from(e: bson::oid::Error) -> Self {
        Error::BsonOid(e)
    }
}

impl From<mongodb::error::Error> for Error {
    fn from(e: mongodb::error::Error) -> Self {
        Error::MongoDb(e)
    }
}

type Result<T> = std::result::Result<T, Error>;

pub struct Database {
    rules: Collection<Rule>,
    authentication: Collection<TokenData>,
}

impl Database {
    pub async fn new() -> Result<Self> {
        // Parse a connection string into an options struct.
        let start = Instant::now();
        info!("Connecting to MongoDB...");
        let mut options = ClientOptions::parse_with_resolver_config(
            &config::mongodb_url(),
            ResolverConfig::cloudflare(),
        )
        .await?;

        // Manually set an option.
        options.app_name = Some("ledger-backend".to_string());

        let client = Client::with_options(options)?;
        let database = client.database("ledger");
        let rules = database.collection_with_type::<Rule>("rules");
        let authentication = database.collection_with_type::<TokenData>("auth");

        info!("Connected to MongoDB! This took {:?}", start.elapsed());

        let db = Database {
            rules,
            authentication,
        };

        Ok(db)
    }

    // RULES

    pub async fn create_or_update_rule(&self, rule: Rule) -> Result<UpdateResult> {
        let opts = UpdateOptions::builder().upsert(true).build();
        let update = UpdateModifications::Document(bson::to_document(&rule)?);
        let result = self
            .rules
            .update_one(
                doc! {"_id": &rule.id.unwrap_or_default()},
                update,
                Some(opts),
            )
            .await?;
        Ok(result)
    }

    pub async fn get_all_rules(&self, filter_by_importer_id: Option<&str>) -> Result<Vec<Rule>> {
        let options = FindOptions::builder().sort(doc!["priority": 1]).build();
        Ok(self
            .rules
            .find(
                filter_by_importer_id.map(|f| doc!["importerId": f]),
                Some(options),
            )
            .await?
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .flatten()
            .collect())
    }

    pub async fn get_rule(&self, rule_id: &str) -> Result<Option<Rule>> {
        Ok(self
            .rules
            .find_one(doc!["_id": ObjectId::with_string(rule_id)?], None)
            .await?)
    }

    pub async fn delete_rule(&self, rule_id: &str) -> Result<DeleteResult> {
        Ok(self
            .rules
            .delete_one(doc!["_id": ObjectId::with_string(rule_id)?], None)
            .await?)
    }

    // AUTH

    pub async fn get_auth(&self) -> Result<Option<TokenData>> {
        let start = Instant::now();

        let doc = self.authentication.find_one(None, None).await?;

        info!("Fetch auth from MongoDB took {:?}", start.elapsed());
        Ok(doc)
    }

    pub async fn set_auth(&self, auth: Option<TokenData>) -> Result<()> {
        if let Some(auth) = auth {
            self.authentication.insert_one(auth, None).await?;
        } else {
            self.authentication.delete_one(doc![], None).await?;
        }
        Ok(())
    }
}
