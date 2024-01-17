use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

use echoma::{
    llama::{options::PredictOptions, LOCAL_LLAMA},
    Result,
};

#[tokio::main]
pub async fn main() -> Result<()> {
    let counter = Arc::new(Mutex::new(0));
    let counter_clone = counter.clone();
    let predict_options = PredictOptions {
        token_callback: Some(Box::new(move |token| {
            let mut count = counter_clone.lock().unwrap();
            *count += 1;
            print!("{}", token);
            true
        })),
        ..Default::default()
    };

    let start_time = Instant::now();
    LOCAL_LLAMA
        .get()
        .await
        .predict(
            "Instruct: Give me some tips for trip in China.\nOutput:".into(),
            predict_options,
        )
        .unwrap();
    let duration = Instant::now().duration_since(start_time);
    println!(
        "Tokens: {}, time: {}",
        counter.clone().lock().unwrap(),
        duration.as_millis()
    );
    Ok(())
}
