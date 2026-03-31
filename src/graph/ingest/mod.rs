pub mod ingestor;

pub mod azure;
pub mod tailscale;

pub(crate) const CREATE_CONSTRAINT_QUERY: &str =
    "CREATE CONSTRAINT IF NOT EXISTS FOR (n:{}) REQUIRE n.{} IS UNIQUE ";
