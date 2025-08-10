use crate::database::get_pool_sync;
use sqlx::{Postgres, Transaction};

#[derive(Debug)]
pub struct DatabaseTransaction;

impl DatabaseTransaction {
    /// Run a closure inside a transaction
    pub async fn run<T, F>(f: F) -> Result<T, crate::error::AppError>
    where
        F: for<'a> FnOnce(
            &'a mut Transaction<'_, Postgres>,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<T, crate::error::AppError>> + Send + 'a>,
        >,
        T: Send,
    {
        let pool = get_pool_sync();
        let mut tx = pool.begin().await.map_err(crate::error::AppError::from)?;

        match f(&mut tx).await {
            Ok(value) => {
                tx.commit().await.map_err(crate::error::AppError::from)?;
                Ok(value)
            }
            Err(err) => {
                log::warn!("Transaction failed with error: {}, rolling back", err);
                if let Err(rollback_err) = tx.rollback().await {
                    log::error!(
                        "Rollback failed after error (orig: {}, rollback: {})",
                        err,
                        rollback_err
                    );
                }
                Err(err)
            }
        }
    }
}
