use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::fmt::Debug;

type Id = i64;

#[async_trait]
pub trait Insert {
    type Output;

    async fn insert(&self, pool: PgPool) -> sqlx::Result<Self::Output>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    Pair(Pair),
}

#[derive(sqlx::Type, Debug, Clone, Copy, Serialize, Deserialize)]
#[sqlx(type_name = "side", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub exchange: String,
    pub market: String,
    pub side: Side,
    pub size: Decimal,
    pub price: Decimal,
    pub date: DateTime<Utc>,
    pub bot: String,
}

impl Trade {
    pub fn balance(&self) -> Decimal {
        match self.side {
            Side::Buy => -self.size,
            Side::Sell => self.size,
        }
    }
}

#[async_trait]
impl Insert for Trade {
    type Output = Id;

    async fn insert(&self, pool: PgPool) -> sqlx::Result<Self::Output> {
        let trade_id = sqlx::query!(
            r#"
            INSERT INTO trades (exchange, market, side, size, price, date, bot)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id"#,
            self.exchange,
            self.market,
            self.side as Side,
            self.size,
            self.price,
            self.date,
            self.bot,
        )
        .fetch_one(&pool)
        .await?
        .id;

        Ok(trade_id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub enter: Trade,
    pub exit: Trade,
}

impl Position {
    pub fn balance(&self) -> Decimal {
        self.enter.balance() + self.exit.balance()
    }
}

#[async_trait]
impl Insert for Position {
    type Output = Id;

    async fn insert(&self, pool: PgPool) -> sqlx::Result<Self::Output> {
        let enter_id = self.enter.insert(pool.clone()).await?;
        let exit_id = self.exit.insert(pool.clone()).await?;

        let position_id = sqlx::query!(
            r#"
            INSERT INTO positions (enter, exit)
            VALUES ($1, $2)
            RETURNING id"#,
            enter_id,
            exit_id,
        )
        .fetch_one(&pool)
        .await?
        .id;

        Ok(position_id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pair {
    pub long: Position,
    pub short: Position,
}

impl Pair {
    pub fn balance(&self) -> Decimal {
        self.long.balance() + self.short.balance()
    }
}

#[async_trait]
impl Insert for Pair {
    type Output = Id;

    async fn insert(&self, pool: PgPool) -> sqlx::Result<Self::Output> {
        let long_id = self.long.insert(pool.clone()).await?;
        let short_id = self.short.insert(pool.clone()).await?;

        let pair_id = sqlx::query!(
            r#"
            INSERT INTO positions (enter, exit)
            VALUES ($1, $2)
            RETURNING id"#,
            long_id,
            short_id,
        )
        .fetch_one(&pool)
        .await?
        .id;

        Ok(pair_id)
    }
}
