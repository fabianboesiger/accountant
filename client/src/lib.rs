use std::sync::Arc;

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use async_trait::async_trait;
use shared::{Side, TradeId, PositionId, PairId};
use sqlx::PgPool;

#[async_trait]
pub trait Trade: Send + Sync + 'static {
    fn exchange(&self) -> String;
    fn market(&self) -> String;
    fn bot(&self) -> String;
    fn size(&self) -> Decimal;
    fn price(&self) -> Decimal;
    fn side(&self) -> Side;
    fn date(&self) -> DateTime<Utc>;

    async fn insert(&self, pool: Arc<PgPool>) -> sqlx::Result<TradeId> {
        let trade_id = sqlx::query!(r#"
            INSERT INTO trades (exchange, market, side, size, price, date, bot)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id"#,
            self.exchange(),
            self.market(),
            self.side() as Side,
            self.size(),
            self.price(),
            self.date(),
            self.bot(),
        )
        .fetch_one(&*pool)
        .await?
        .id;

        Ok(trade_id)
    }
}

#[async_trait]
pub trait Position<T>: Send + Sync + 'static
where
    T: Trade,
{
    fn enter(&self) -> T;
    fn exit(&self) -> T;

    fn exchange(&self) -> String {
        let enter = self.enter().exchange();
        let exit = self.exit().exchange();
        assert_eq!(enter, exit);
        enter
    }

    fn market(&self) -> String {
        let enter = self.enter().market();
        let exit = self.exit().market();
        assert_eq!(enter, exit);
        enter
    }

    fn bot(&self) -> String {
        let enter = self.enter().bot();
        let exit = self.exit().bot();
        assert_eq!(enter, exit);
        enter
    }

    async fn insert(&self, pool: Arc<PgPool>) -> sqlx::Result<PositionId> {
        // Assertions.
        self.exchange();
        self.market();
        self.bot();

        let enter_id = self.enter().insert(pool.clone()).await?;
        let exit_id = self.exit().insert(pool.clone()).await?;
        
        let position_id = sqlx::query!(r#"
            INSERT INTO positions (enter, exit)
            VALUES ($1, $2)
            RETURNING id"#,
            enter_id,
            exit_id,
        )
        .fetch_one(&*pool)
        .await?
        .id;

        Ok(position_id)
    }
}

#[async_trait]
pub trait Pair<P, T>: Send + Sync + 'static
where
    P: Position<T>,
    T: Trade,
{
    fn long(&self) -> P;
    fn short(&self) -> P;

    fn bot(&self) -> String {
        let long = self.long().bot();
        let short = self.short().bot();
        assert_eq!(long, short);
        long
    }

    async fn insert(&self, pool: Arc<PgPool>) -> sqlx::Result<PairId> {
        // Assertions.
        self.bot();

        let long_id = self.long().insert(pool.clone()).await?;
        let short_id = self.short().insert(pool.clone()).await?;
        
        let pair_id = sqlx::query!(r#"
            INSERT INTO pairs (long, short)
            VALUES ($1, $2)
            RETURNING id"#,
            long_id,
            short_id,
        )
        .fetch_one(&*pool)
        .await?
        .id;

        Ok(pair_id)
    }
}

pub struct Accountant {
    pool: Arc<PgPool>
}

impl Accountant {
    pub async fn new() -> Self {
        dotenv::dotenv().ok();
        let pool = PgPool::connect(&std::env::var("DATABASE_URL").expect("DATABASE_URL not set.")).await.unwrap();
        
        Self {
            pool: Arc::new(pool)
        }
    }

    // Inserts a pair trade into the database
    // in a non-blocking manner.
    pub fn insert_pair<Q, P, T>(&self, pair: Q)
    where
        Q: Pair<P, T>,
        P: Position<T>,
        T: Trade
    {
        let pool = self.pool.clone();
        tokio::spawn(async move {
            pair.insert(pool).await.unwrap();
        });
    }
}

#[cfg(test)]
mod tests {
    use crate::Accountant;

    #[tokio::test]
    async fn db_connect() {
        let _ = Accountant::new().await;
    }
}
