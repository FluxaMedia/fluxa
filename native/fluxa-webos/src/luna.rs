use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{Receiver, channel};
use std::time::Duration;

pub fn call(uri: &str, payload: &str, timeout: Duration) -> Option<String> {
    let (uri, payload) = (uri.to_owned(), payload.to_owned());
    let (tx, rx) = channel();
    std::thread::spawn(move || {
        let output = Command::new("luna-send")
            .args(["-n", "1", "-f", &uri, &payload])
            .output();
        let _ = tx.send(output);
    });
    match rx.recv_timeout(timeout) {
        Ok(Ok(output)) => Some(String::from_utf8_lossy(&output.stdout).into_owned()),
        _ => None,
    }
}

pub struct Subscription {
    child: Child,
}

pub fn subscribe(uri: &str, payload: &str) -> Option<(Subscription, Receiver<String>)> {
    let mut child = Command::new("luna-send")
        .args(["-i", uri, payload])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let stdout = child.stdout.take()?;
    let (tx, lines) = channel();
    std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
            if tx.send(line).is_err() {
                break;
            }
        }
    });
    Some((Subscription { child }, lines))
}

impl Drop for Subscription {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
