use std::future::Future;
use std::pin::Pin;

pub async fn retry_on_err<F, T, E>(
    mut f: F,
    retries: u32,
) -> Result<T, E>
where
    F: FnMut(u32) -> Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'static>>,
{
    let mut last_err: Option<E> = None;

    for attempt in 1..=retries {
        match f(attempt).await {
            Ok(result) => return Ok(result),
            Err(err) => {
                last_err = Some(err);
            }
        }
    }

    Err(last_err.expect("attempts were not made"))
}