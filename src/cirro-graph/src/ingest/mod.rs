pub mod ingestor;

pub mod azure;
pub mod tailscale;

const CREATE_CONSTRAINT_QUERY: &str =
    "CREATE CONSTRAINT IF NOT EXISTS FOR (n:{}) REQUIRE n.id IS UNIQUE ";

const CREATE_INDEX_QUERY: &str = "CREATE TEXT INDEX {}_id IF NOT EXISTS FOR (n:{}) ON (n.id)";
