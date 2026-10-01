use std::sync::mpsc::{Receiver, channel};
use std::time::Duration;

pub fn fetch_text(url: &str) -> Receiver<Option<String>> {
    let (sender, receiver) = channel();
    let url = url.to_owned();
    std::thread::spawn(move || {
        let text = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .ok()
            .and_then(|runtime| {
                runtime.block_on(async {
                    let client = reqwest::Client::builder()
                        .timeout(Duration::from_secs(15))
                        .build()
                        .ok()?;
                    client.get(&url).send().await.ok()?.text().await.ok()
                })
            });
        let _ = sender.send(text);
    });
    receiver
}
