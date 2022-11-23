use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Events {
    pub events: Vec<Event>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Event {
    pub cause: Cause,
    pub changes: Vec<Change>,
    pub occurrence: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub enum Cause {
    Deposit,
    Withdraw,
    Rebalance,
    Fee,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Change {
    #[serde(rename = "type")]
    pub change_type: ChangeType,
    pub size: f64,
    #[serde(rename = "securityDefinition")]
    pub security_definition: SecurityDefinition,
    pub price: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub enum ChangeType {
    Cash,
    Security,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub enum SecurityDefinition {
    TSWE,
    TRET,
    TGBT,
    TCBT,
    None,
}
