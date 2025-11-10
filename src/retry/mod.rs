

pub async fn retry_on_err(f: impl AsyncFn(u32) -> bool, retries: u32) -> bool{

    for attempt in 1..= retries{
        let res = f(attempt).await;
        if res{
            return res;
        }
    }

    false
}


