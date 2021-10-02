use rust_decimal::Decimal;
use chrono::{DateTime, Utc};

#[derive(sqlx::Type, Debug, Clone, Copy)]
#[sqlx(type_name = "side", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Side {
    Buy,
    Sell,
}

pub type TradeId = i64;

#[derive(Debug, Clone)]
pub struct Trade {
    pub id: TradeId,
    pub exchange: String,
    pub market: String,
    pub side: Side,
    pub size: Decimal,
    pub price: Decimal,
    pub date: DateTime<Utc>,
    pub bot: String,
}

pub type PositionId = i64;

#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub id: PositionId,
    pub enter: TradeId,
    pub exit: TradeId,
}

pub type PairId = i64;

#[derive(Debug, Clone, Copy)]
pub struct Pair {
    pub id: PairId,
    pub long: PositionId,
    pub short: PositionId,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
