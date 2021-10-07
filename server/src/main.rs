use rust_decimal::prelude::Zero;
use rust_decimal::Decimal;
use shared::*;
use sqlx::PgPool;
use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

type AnyError = Box<dyn std::error::Error + Send + Sync>;

#[tokio::main]
async fn main() -> Result<(), AnyError> {
    dotenv::dotenv().ok();
    let (sender, receiver) = mpsc::unbounded_channel();
    let pool = PgPool::connect(&std::env::var("DATABASE_URL").unwrap()).await?;

    tokio::try_join!(listen(sender), handle(receiver, pool),)?;
    Ok(())
}

async fn listen(sender: UnboundedSender<Message>) -> Result<(), AnyError> {
    let listener = TcpListener::bind("127.0.0.1:5000").await?;

    loop {
        let (stream, _) = listener.accept().await?;
        match receive(stream, sender.clone()).await {
            Ok(()) => log::warn!("Stream ended."),
            Err(err) => log::error!("Got receiver error {:?}", err),
        };
    }
}

async fn receive(mut stream: TcpStream, sender: UnboundedSender<Message>) -> Result<(), AnyError> {
    let mut buffer = Vec::new();
    stream.read_to_end(&mut buffer).await?;
    let message = bincode::deserialize(&buffer)?;
    sender.send(message)?;
    Ok(())
}

async fn handle(mut receiver: UnboundedReceiver<Message>, pool: PgPool) -> Result<(), AnyError> {
    while let Some(message) = receiver.recv().await {
        match message {
            Message::Pair(pair) => {
                pair.insert(pool.clone()).await?;

                let mut transaction = pool.begin().await?;
                let pair_profits = pair.balance();
                let total = sqlx::query!(
                    r#"
                        SELECT SUM(balance) AS total
                        FROM accounts"#
                )
                .fetch_one(&mut transaction)
                .await?
                .total
                .unwrap();

                let accounts = sqlx::query!(
                    r#"
                        SELECT id, balance, fee
                        FROM accounts"#
                )
                .fetch_all(&mut transaction)
                .await?;

                for account in accounts {
                    let account_profits = (account.balance / total) * pair_profits;
                    let fee_account = 0;
                    let fee = 
                        if account_profits > Decimal::zero() && account.id != fee_account {
                            account.fee * account_profits
                        } else {
                            Decimal::zero()
                        };
                    let feed_account_profits = account_profits - fee;

                    sqlx::query!(
                        r#"
                        INSERT INTO payments (account, amount, payment_type)
                        VALUES ($1, $2, 'INVESTMENT')"#,
                        account.id,
                        feed_account_profits
                    )
                    .execute(&mut transaction)
                    .await?;

                    sqlx::query!(
                        r#"
                        UPDATE accounts
                        SET balance = balance + $2
                        WHERE id = $1"#,
                        account.id,
                        feed_account_profits
                    )
                    .execute(&mut transaction)
                    .await?;

                    if fee > Decimal::zero() {
                        sqlx::query!(
                            r#"
                            INSERT INTO payments (account, amount, payment_type)
                            VALUES ($1, $2, 'FEE')"#,
                            fee_account,
                            fee
                        )
                        .execute(&mut transaction)
                        .await?;
    
                        sqlx::query!(
                            r#"
                            UPDATE accounts
                            SET balance = balance + $2
                            WHERE id = $1"#,
                            fee_account,
                            fee
                        )
                        .execute(&mut transaction)
                        .await?;
                    }
                    
                }

                transaction.commit().await?;
            }
        }
    }
    Ok(())
}
