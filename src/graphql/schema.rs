use crate::models::Transfer;

use async_graphql::*;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use serde::{Deserialize, Serialize};
use tracing::info;

pub struct QueryRoot;

#[derive(Serialize, Deserialize)]
pub struct BlockHeightFilter {
    ge: Option<u64>,
    le: Option<u64>,
}

scalar!(BlockHeightFilter);

#[Object]
impl QueryRoot {
    pub async fn transfer<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        filter: Option<BlockHeightFilter>,
        block_height: Option<u64>,
    ) -> Result<Vec<Transfer>> {
        let data = ctx.data()?;
        let pool: &Pool<SqliteConnectionManager> = data;
        let con = pool.get()?;

        if let Some(height) = block_height {
            let mut stmt =
                con.prepare("SELECT rowid, * FROM Transfers WHERE block_height = (?)")?;

            info!("Fetching block with block_height {}", height);

            let transfers = stmt.query_map([height], |row| {
                print!("te");
                let x = Transfer::try_from(row);
                println!("{:?}", x);
                x
            })?;

            return Ok(transfers.flatten().collect());
        }

        if let Some(filter) = filter {
            match (filter.ge, filter.le) {
                (Some(x), Some(y)) => {
                    let mut stmt = con.prepare(
                        "SELECT * FROM Transfers WHERE block_height > (?) AND block_height < (?)",
                    )?;
                    let transfers = stmt.query_map([x, y], |row| Transfer::try_from(row))?;

                    return Ok(transfers.flatten().collect());
                }
                (Some(x), None) => {
                    let mut stmt =
                        con.prepare("SELECT * FROM Transfers WHERE block_height > (?)")?;
                    let transfers = stmt.query_map([x], |row| Transfer::try_from(row))?;

                    return Ok(transfers.flatten().collect());
                }
                (None, Some(y)) => {
                    let mut stmt =
                        con.prepare("SELECT * FROM Transfers WHERE block_height < (?)")?;
                    let transfers = stmt.query_map([y], |row| Transfer::try_from(row))?;

                    return Ok(transfers.flatten().collect());
                }
                _ => (),
            };
        }

        println!("{:?}", block_height);
        Ok(Vec::new())
    }
}
