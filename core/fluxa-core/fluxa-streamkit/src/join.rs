use std::pin::Pin;
use std::task::{Context, Poll};

pub struct JoinAll<F: Future> {
    futures: Vec<Option<Pin<Box<F>>>>,
    done: Vec<Option<F::Output>>,
}

pub fn join_all<F: Future>(futures: impl IntoIterator<Item = F>) -> JoinAll<F> {
    let futures: Vec<_> = futures.into_iter().map(|f| Some(Box::pin(f))).collect();
    let done = futures.iter().map(|_| None).collect();
    JoinAll { futures, done }
}

impl<F: Future> Unpin for JoinAll<F> {}

impl<F: Future> Future for JoinAll<F> {
    type Output = Vec<F::Output>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        let mut pending = false;
        for (slot, out) in this.futures.iter_mut().zip(this.done.iter_mut()) {
            if let Some(f) = slot {
                match f.as_mut().poll(cx) {
                    Poll::Ready(v) => {
                        *out = Some(v);
                        *slot = None;
                    }
                    Poll::Pending => pending = true,
                }
            }
        }
        if pending {
            return Poll::Pending;
        }
        Poll::Ready(this.done.iter_mut().filter_map(Option::take).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::task::Waker;

    struct Yield(Cell<u32>);

    impl Future for Yield {
        type Output = u32;

        fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<u32> {
            let left = self.0.get();
            if left == 0 {
                return Poll::Ready(7);
            }
            self.0.set(left - 1);
            Poll::Pending
        }
    }

    fn drive<T>(fut: impl Future<Output = T>) -> T {
        let mut fut = Box::pin(fut);
        let mut cx = Context::from_waker(Waker::noop());
        loop {
            if let Poll::Ready(v) = fut.as_mut().poll(&mut cx) {
                return v;
            }
        }
    }

    #[test]
    fn results_keep_input_order_even_when_finishing_out_of_order() {
        let out = drive(join_all(vec![
            Box::pin(async { Yield(Cell::new(3)).await + 1 }) as Pin<Box<dyn Future<Output = u32>>>,
            Box::pin(async { 1 }),
            Box::pin(async { Yield(Cell::new(1)).await + 2 }),
        ]));
        assert_eq!(out, vec![8, 1, 9]);
    }

    #[test]
    fn empty_input_completes_immediately() {
        let out: Vec<u32> = drive(join_all(Vec::<std::future::Ready<u32>>::new()));
        assert!(out.is_empty());
    }
}
