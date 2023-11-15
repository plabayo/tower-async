use super::{Action, Attempt, Policy};
use http::Request;

/// A redirection [`Policy`] that combines the results of two `Policy`s.
///
/// See [`PolicyExt::or`][super::PolicyExt::or] for more details.
#[derive(Clone, Copy, Debug, Default)]
pub struct Or<A, B> {
    a: A,
    b: B,
}

impl<A, B> Or<A, B> {
    pub(crate) fn new<Bd, E>(a: A, b: B) -> Self
    where
        A: Policy<Bd, E>,
        B: Policy<Bd, E>,
    {
        Or { a, b }
    }
}

impl<Bd, E, A, B> Policy<Bd, E> for Or<A, B>
where
    A: Policy<Bd, E>,
    B: Policy<Bd, E>,
{
    fn redirect(&self, attempt: &Attempt<'_>) -> Result<Action, E> {
        match self.a.redirect(attempt) {
            Ok(Action::Stop) | Err(_) => self.b.redirect(attempt),
            a => a,
        }
    }

    fn on_request(&self, request: &mut Request<Bd>) {
        self.a.on_request(request);
        self.b.on_request(request);
    }

    fn clone_body(&self, body: &Bd) -> Option<Bd> {
        self.a.clone_body(body).or_else(|| self.b.clone_body(body))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::AtomicBool;

    use super::*;
    use http::Uri;

    struct Taint<P> {
        policy: P,
        _used: AtomicBool,
    }

    impl<P> Taint<P> {
        fn new(policy: P) -> Self {
            Taint {
                policy,
                _used: AtomicBool::new(false),
            }
        }

        fn used(&self) -> bool {
            self._used.load(std::sync::atomic::Ordering::SeqCst)
        }
    }

    impl<B, E, P> Policy<B, E> for Taint<P>
    where
        P: Policy<B, E>,
    {
        fn redirect(&self, attempt: &Attempt<'_>) -> Result<Action, E> {
            self._used.store(true, std::sync::atomic::Ordering::SeqCst);
            self.policy.redirect(attempt)
        }
    }

    #[test]
    fn redirect() {
        let attempt = Attempt {
            status: Default::default(),
            location: &Uri::from_static("*"),
            previous: &Uri::from_static("*"),
        };

        let a = Taint::new(Action::Follow);
        let b = Taint::new(Action::Follow);
        let policy = Or::new::<(), ()>(&a, &b);
        assert!(Policy::<(), ()>::redirect(&policy, &attempt)
            .unwrap()
            .is_follow());
        assert!(a.used());
        assert!(!b.used()); // short-circuiting

        let a = Taint::new(Action::Stop);
        let b = Taint::new(Action::Follow);
        let policy = Or::new::<(), ()>(&a, &b);
        assert!(Policy::<(), ()>::redirect(&policy, &attempt)
            .unwrap()
            .is_follow());
        assert!(a.used());
        assert!(b.used());

        let a = Taint::new(Action::Follow);
        let b = Taint::new(Action::Stop);
        let policy = Or::new::<(), ()>(&a, &b);
        assert!(Policy::<(), ()>::redirect(&policy, &attempt)
            .unwrap()
            .is_follow());
        assert!(a.used());
        assert!(!b.used());

        let a = Taint::new(Action::Stop);
        let b = Taint::new(Action::Stop);
        let policy = Or::new::<(), ()>(&a, &b);
        assert!(Policy::<(), ()>::redirect(&policy, &attempt)
            .unwrap()
            .is_stop());
        assert!(a.used());
        assert!(b.used());
    }
}
