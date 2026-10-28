use std::pin::Pin;
use std::task::Poll;

use serde::{Deserialize, Serialize};

use crate::expr::{Compiled, Scope, Value};
use crate::fetch::{
    AddonEndpoint, AddonReport, AddonResult, Outcome, Transport, cut_off, fetch_addon,
};
use crate::types::{ParsedStream, StreamType};

pub trait Clock {
    fn now_ms(&self) -> i64;
    fn sleep(&self, ms: u64) -> impl Future<Output = ()>;
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Behaviour {
    #[default]
    Parallel,
    Sequential,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OnFalse {
    #[default]
    Stop,
    Skip,
    IncludeFinished,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Group {
    pub addons: Vec<String>,
    pub condition: Option<String>,
    pub behaviour: Behaviour,
    pub on_false: OnFalse,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Plan {
    pub groups: Vec<Group>,
    pub exit: Option<String>,
}

#[derive(Debug, Default)]
pub struct PlanOutput {
    pub groups: Vec<Vec<ParsedStream>>,
    pub reports: Vec<AddonReport>,
    pub exited_early: bool,
}

type Job<'a> = Pin<Box<dyn Future<Output = AddonResult> + 'a>>;

const TICK_SLACK_MS: i64 = 50;

fn counts(s: &ParsedStream) -> bool {
    !matches!(
        s.stream_type,
        StreamType::Error | StreamType::Info | StreamType::Statistic
    )
}

fn assign(endpoints: &[AddonEndpoint], plan: &Plan) -> Vec<Vec<usize>> {
    if plan.groups.is_empty() {
        return vec![(0..endpoints.len()).collect()];
    }
    let mut out: Vec<Vec<usize>> = vec![Vec::new(); plan.groups.len()];
    for (i, e) in endpoints.iter().enumerate() {
        let slot = plan
            .groups
            .iter()
            .position(|g| g.addons.iter().any(|a| *a == e.addon.instance_id))
            .unwrap_or(0);
        out[slot].push(i);
    }
    out
}

struct Progress<'a> {
    exit: Option<Compiled>,
    thresholds: Vec<i64>,
    start: i64,
    scope: &'a Scope,
    total: usize,
    queried: Vec<String>,
}

impl Progress<'_> {
    fn scope(&self, now: i64) -> Scope {
        let mut s = self.scope.clone();
        s.set("totalStreams", Value::Num(self.total as f64));
        s.set("totalTimeTaken", Value::Num((now - self.start) as f64));
        s.set(
            "queriedAddons",
            Value::List(self.queried.iter().cloned().map(Value::Str).collect()),
        );
        s
    }

    fn should_exit(&self, now: i64) -> bool {
        self.exit
            .as_ref()
            .is_some_and(|e| e.condition(&self.scope(now)))
    }

    fn next_tick(&self, now: i64) -> Option<u64> {
        let elapsed = now - self.start;
        self.thresholds
            .iter()
            .map(|t| t + TICK_SLACK_MS)
            .filter(|t| *t > elapsed)
            .min()
            .and_then(|t| u64::try_from(t - elapsed).ok())
    }
}

enum Event {
    Done(usize, Box<AddonResult>),
    Tick,
}

async fn run_batch<'a, T: Transport, C: Clock>(
    transport: &'a T,
    clock: &'a C,
    endpoints: &'a [AddonEndpoint],
    batch: &[usize],
    media_type: &'a str,
    id: &'a str,
    progress: &mut Progress<'_>,
) -> (Vec<(usize, AddonResult)>, Vec<usize>) {
    let mut jobs: Vec<Option<Job<'a>>> = batch
        .iter()
        .map(|&i| {
            let fut = async move {
                let started = clock.now_ms();
                let mut r = fetch_addon(transport, &endpoints[i], media_type, id, started).await;
                r.report.latency_ms = u64::try_from(clock.now_ms() - started).unwrap_or(0);
                r
            };
            Some(Box::pin(fut) as Job<'a>)
        })
        .collect();
    let mut finished = Vec::new();
    loop {
        if jobs.iter().all(Option::is_none) {
            return (finished, Vec::new());
        }
        let now = clock.now_ms();
        let mut timer: Option<Pin<Box<dyn Future<Output = ()> + '_>>> = progress
            .next_tick(now)
            .map(|ms| Box::pin(clock.sleep(ms)) as Pin<Box<dyn Future<Output = ()> + '_>>);
        let event = std::future::poll_fn(|cx| {
            for (slot, &endpoint) in jobs.iter_mut().zip(batch) {
                if let Some(job) = slot {
                    if let Poll::Ready(r) = job.as_mut().poll(cx) {
                        *slot = None;
                        return Poll::Ready(Event::Done(endpoint, Box::new(r)));
                    }
                }
            }
            if let Some(t) = timer.as_mut() {
                if t.as_mut().poll(cx).is_ready() {
                    return Poll::Ready(Event::Tick);
                }
            }
            Poll::Pending
        })
        .await;
        match event {
            Event::Done(i, result) => {
                progress.total += result.streams.iter().filter(|s| counts(s)).count();
                if result.report.outcome == Outcome::Merged {
                    progress.queried.push(endpoints[i].addon.name.clone());
                }
                finished.push((i, *result));
            }
            Event::Tick => {}
        }
        if progress.should_exit(clock.now_ms()) {
            let abandoned = jobs
                .iter()
                .zip(batch)
                .filter(|(j, _)| j.is_some())
                .map(|(_, &i)| i)
                .collect();
            return (finished, abandoned);
        }
    }
}

pub async fn run_plan<T: Transport, C: Clock>(
    transport: &T,
    clock: &C,
    endpoints: &[AddonEndpoint],
    plan: &Plan,
    media_type: &str,
    id: &str,
    scope: &Scope,
) -> PlanOutput {
    let start = clock.now_ms();
    let exit = plan.exit.as_deref().and_then(|s| Compiled::compile(s).ok());
    let thresholds = exit
        .as_ref()
        .map(|e| e.time_thresholds().into_iter().map(|t| t as i64).collect())
        .unwrap_or_default();
    let mut progress = Progress {
        exit,
        thresholds,
        start,
        scope,
        total: 0,
        queried: Vec::new(),
    };
    let assigned = assign(endpoints, plan);
    let mut reports: Vec<Option<AddonReport>> = vec![None; endpoints.len()];
    let mut out = PlanOutput::default();
    let mut previous: Vec<ParsedStream> = Vec::new();
    let mut previous_took = 0i64;
    'groups: for (gi, members) in assigned.iter().enumerate() {
        let group = plan.groups.get(gi).cloned().unwrap_or_default();
        if gi > 0 {
            if let Some(cond) = group
                .condition
                .as_deref()
                .and_then(|s| Compiled::compile(s).ok())
            {
                let mut s = progress.scope(clock.now_ms());
                s.set("previousGroupTimeTaken", Value::Num(previous_took as f64));
                if !cond.condition_over(&previous, &s) {
                    match group.on_false {
                        OnFalse::Skip => {
                            out.groups.push(Vec::new());
                            continue 'groups;
                        }
                        OnFalse::Stop | OnFalse::IncludeFinished => break 'groups,
                    }
                }
            }
        }
        let group_start = clock.now_ms();
        let batches: Vec<Vec<usize>> = match group.behaviour {
            Behaviour::Parallel => vec![members.clone()],
            Behaviour::Sequential => members.iter().map(|i| vec![*i]).collect(),
        };
        let mut collected: Vec<(usize, AddonResult)> = Vec::new();
        let mut exited = false;
        for batch in batches.iter().filter(|b| !b.is_empty()) {
            if exited {
                break;
            }
            let (done, abandoned) = run_batch(
                transport,
                clock,
                endpoints,
                batch,
                media_type,
                id,
                &mut progress,
            )
            .await;
            collected.extend(done);
            if !abandoned.is_empty() || progress.should_exit(clock.now_ms()) {
                exited = true;
            }
            for i in abandoned {
                reports[i] = Some(cut_off(&endpoints[i], true));
            }
        }
        collected.sort_by_key(|(i, _)| *i);
        let mut streams = Vec::new();
        for (i, r) in collected {
            reports[i] = Some(r.report);
            streams.extend(r.streams);
        }
        previous = streams.iter().filter(|s| counts(s)).cloned().collect();
        previous_took = clock.now_ms() - group_start;
        out.groups.push(streams);
        if exited {
            out.exited_early = true;
            break;
        }
    }
    out.reports = reports
        .into_iter()
        .zip(endpoints)
        .map(|(r, e)| r.unwrap_or_else(|| cut_off(e, false)))
        .collect();
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fetch::{TransportError, TransportResponse};
    use crate::types::AddonRef;
    use std::cell::Cell;
    use std::task::{Context, Waker};
    use std::time::Duration;

    fn block_on<T>(fut: impl Future<Output = T>) -> T {
        let mut fut = Box::pin(fut);
        let mut cx = Context::from_waker(Waker::noop());
        loop {
            if let Poll::Ready(v) = fut.as_mut().poll(&mut cx) {
                return v;
            }
        }
    }

    struct Ticking {
        now: Cell<i64>,
        step: i64,
    }

    impl Clock for Ticking {
        fn now_ms(&self) -> i64 {
            let n = self.now.get();
            self.now.set(n + self.step);
            n
        }

        async fn sleep(&self, _ms: u64) {}
    }

    struct Routes;

    impl Transport for Routes {
        async fn get(&self, url: &str, _t: Duration) -> Result<TransportResponse, TransportError> {
            if url.contains("hang.example") {
                std::future::pending::<()>().await;
            }
            let streams = if url.contains("empty.example") {
                "[]".to_string()
            } else {
                r#"[{"url":"https://x/a.mkv","title":"Movie.2020.1080p.WEB-DL-GRP"}]"#.to_string()
            };
            Ok(TransportResponse {
                status: 200,
                body: format!(r#"{{"streams":{streams}}}"#),
            })
        }
    }

    fn endpoint(name: &str, host: &str) -> AddonEndpoint {
        AddonEndpoint {
            addon: AddonRef {
                instance_id: name.into(),
                preset_id: "custom".into(),
                name: name.into(),
                ..Default::default()
            },
            manifest_url: format!("https://{host}/manifest.json"),
            timeout: Duration::from_secs(5),
            media_types: Vec::new(),
        }
    }

    fn group(addons: &[&str], condition: Option<&str>, on_false: OnFalse) -> Group {
        Group {
            addons: addons.iter().map(|s| s.to_string()).collect(),
            condition: condition.map(String::from),
            behaviour: Behaviour::Parallel,
            on_false,
        }
    }

    fn run(endpoints: &[AddonEndpoint], plan: &Plan, step: i64) -> PlanOutput {
        let clock = Ticking {
            now: Cell::new(0),
            step,
        };
        block_on(run_plan(
            &Routes,
            &clock,
            endpoints,
            plan,
            "movie",
            "tt1",
            &Scope::default(),
        ))
    }

    #[test]
    fn unassigned_addons_land_in_the_first_group() {
        let eps = [endpoint("a", "a.example"), endpoint("b", "b.example")];
        let plan = Plan {
            groups: vec![
                group(&["a"], None, OnFalse::Stop),
                group(&[], None, OnFalse::Stop),
            ],
            exit: None,
        };
        let out = run(&eps, &plan, 1);
        assert_eq!(out.groups.len(), 2);
        assert_eq!(out.groups[0].len(), 2);
        assert!(out.groups[1].is_empty());
    }

    #[test]
    fn false_condition_stops_remaining_groups() {
        let eps = [endpoint("a", "a.example"), endpoint("b", "b.example")];
        let plan = Plan {
            groups: vec![
                group(&["a"], None, OnFalse::Stop),
                group(&["b"], Some("count(streams) == 0"), OnFalse::Stop),
            ],
            exit: None,
        };
        let out = run(&eps, &plan, 1);
        assert_eq!(out.groups.len(), 1);
        assert_eq!(out.reports[1].outcome, Outcome::NotStarted);
    }

    #[test]
    fn skip_moves_on_to_the_next_group() {
        let eps = [
            endpoint("a", "a.example"),
            endpoint("b", "b.example"),
            endpoint("c", "c.example"),
        ];
        let plan = Plan {
            groups: vec![
                group(&["a"], None, OnFalse::Stop),
                group(&["b"], Some("totalStreams > 5"), OnFalse::Skip),
                group(&["c"], Some("totalStreams >= 1"), OnFalse::Stop),
            ],
            exit: None,
        };
        let out = run(&eps, &plan, 1);
        assert_eq!(out.groups.len(), 3);
        assert!(out.groups[1].is_empty());
        assert_eq!(out.groups[2].len(), 1);
        assert_eq!(out.reports[1].outcome, Outcome::NotStarted);
    }

    #[test]
    fn exit_by_count_abandons_the_rest_of_the_batch() {
        let eps = [endpoint("a", "a.example"), endpoint("b", "b.example")];
        let plan = Plan {
            groups: Vec::new(),
            exit: Some("totalStreams >= 1".into()),
        };
        let out = run(&eps, &plan, 1);
        assert!(out.exited_early);
        assert_eq!(out.reports[0].outcome, Outcome::Merged);
        assert_eq!(out.reports[1].outcome, Outcome::CutOff);
    }

    #[test]
    fn exit_by_time_cuts_off_a_hanging_addon() {
        let eps = [endpoint("a", "a.example"), endpoint("slow", "hang.example")];
        let plan = Plan {
            groups: Vec::new(),
            exit: Some("totalTimeTaken > 500".into()),
        };
        let out = run(&eps, &plan, 300);
        assert!(out.exited_early);
        assert_eq!(out.reports[0].outcome, Outcome::Merged);
        assert_eq!(out.reports[1].outcome, Outcome::CutOff);
        assert_eq!(out.groups[0].len(), 1);
    }

    #[test]
    fn sequential_groups_stop_launching_after_exit() {
        let eps = [endpoint("a", "a.example"), endpoint("b", "b.example")];
        let plan = Plan {
            groups: vec![Group {
                addons: vec!["a".into(), "b".into()],
                behaviour: Behaviour::Sequential,
                ..Default::default()
            }],
            exit: Some("totalStreams >= 1".into()),
        };
        let out = run(&eps, &plan, 1);
        assert_eq!(out.reports[0].outcome, Outcome::Merged);
        assert_eq!(out.reports[1].outcome, Outcome::NotStarted);
    }
}
