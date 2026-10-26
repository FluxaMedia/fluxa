use std::io::Read;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};
use tao::event::{Event, StartCause};
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tao::window::WindowBuilder;
use wry::WebViewBuilder;

pub const ARG: &str = "--cf-solve";
const CHILD_DEADLINE: Duration = Duration::from_secs(16);
const POLL: Duration = Duration::from_millis(400);

pub fn start_pump() {
    fluxa_effects::set_challenge_solver_available(true);
    let _ = std::thread::Builder::new()
        .name("fluxa-cf-solver".to_owned())
        .spawn(|| {
            loop {
                let Some((id, url)) = fluxa_effects::next_challenge_request(1000) else {
                    continue;
                };
                let (cookies, user_agent, expires_at) = solve_in_child(&url).unwrap_or_default();
                fluxa_effects::complete_challenge(id, &cookies, &user_agent, expires_at);
            }
        });
}

fn solve_in_child(url: &str) -> Option<(String, String, u64)> {
    let mut child = Command::new(std::env::current_exe().ok()?)
        .args([ARG, url])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let started = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if started.elapsed() < CHILD_DEADLINE + Duration::from_secs(2) => {
                std::thread::sleep(Duration::from_millis(100));
            }
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
        }
    };
    if !status.success() {
        return None;
    }
    let mut output = String::new();
    child.stdout.take()?.read_to_string(&mut output).ok()?;
    let mut lines = output.lines();
    let user_agent = lines.next()?.trim().to_owned();
    let cookies = lines.next()?.trim().to_owned();
    let expires_at = lines.next().and_then(|line| line.trim().parse().ok());
    Some((cookies, user_agent, expires_at.unwrap_or(0)))
}

pub fn run_child(url: &str) -> ! {
    let event_loop = EventLoopBuilder::new().build();
    let Ok(window) = WindowBuilder::new().with_visible(false).build(&event_loop) else {
        std::process::exit(1)
    };
    let builder = WebViewBuilder::new().with_url(url);
    #[cfg(target_os = "linux")]
    let webview = {
        use tao::platform::unix::WindowExtUnix;
        use wry::WebViewBuilderExtUnix;
        window
            .default_vbox()
            .and_then(|vbox| builder.build_gtk(vbox).ok())
    };
    #[cfg(not(target_os = "linux"))]
    let webview = builder.build(&window).ok();
    let Some(webview) = webview else {
        std::process::exit(1)
    };

    let (agent_tx, agent_rx) = mpsc::channel();
    let started = Instant::now();
    let url = url.to_owned();
    let mut cookies: Option<(String, i64)> = None;
    event_loop.run(move |event, _, flow| {
        *flow = ControlFlow::WaitUntil(Instant::now() + POLL);
        let _window = &window;
        if !matches!(
            event,
            Event::NewEvents(StartCause::ResumeTimeReached { .. })
        ) {
            return;
        }
        if started.elapsed() > CHILD_DEADLINE {
            std::process::exit(1);
        }
        let Some((cookies, expires_at)) = &cookies else {
            let jar = webview.cookies_for_url(&url).unwrap_or_default();
            if let Some(clearance) = jar.iter().find(|cookie| cookie.name() == "cf_clearance") {
                let expires_at = clearance
                    .expires_datetime()
                    .map_or(0, |at| at.unix_timestamp());
                cookies = Some((
                    jar.iter()
                        .map(|cookie| format!("{}={}", cookie.name(), cookie.value()))
                        .collect::<Vec<_>>()
                        .join("; "),
                    expires_at,
                ));
                let tx = agent_tx.clone();
                let _ =
                    webview.evaluate_script_with_callback("navigator.userAgent", move |value| {
                        let _ = tx.send(value.trim_matches('"').to_owned());
                    });
            }
            return;
        };
        if let Ok(agent) = agent_rx.try_recv() {
            println!("{agent}\n{cookies}\n{expires_at}");
            std::process::exit(0);
        }
    })
}
