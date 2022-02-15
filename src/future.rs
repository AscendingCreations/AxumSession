use crate::AxumSession;
use futures::executor::block_on;
use futures_util::ready;
use http::Response;
use pin_project_lite::pin_project;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

// This is a Future which is Ran at the end of a Route to Process whats left over
// or add cookies ETC to the Headers or Update HTML.
pin_project! {
    /// Response future for [`SessionManager`].
    #[derive(Debug)]
    pub struct AxumDatabaseResponseFuture<F> {
        #[pin]
        pub(crate) future: F,
        pub(crate) session: AxumSession,
    }
}

/// This Portion runs when the Route has finished running.
/// It can not See any Extensions for some reason...
impl<F, ResBody, E> Future for AxumDatabaseResponseFuture<F>
where
    F: Future<Output = Result<Response<ResBody>, E>>,
{
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let res = ready!(this.future.poll(cx)?);

        //Check to get the Session itself so it can be Saved to the Database on Response
        //TODO: Find a more Finite way to do this so server is less bogged down?
        let store_ug = this.session.store.inner.upgradable_read();
        if let Some(sess) = store_ug.get(&this.session.id.0.to_string()) {
            let inner = sess.lock();
            let _ = block_on(this.session.store.store_session(inner.clone()));
        }

        Poll::Ready(Ok(res))
    }
}
