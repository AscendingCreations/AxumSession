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
        let future = ready!(this.future.poll(cx))?;

        //Lets lock get a clone of the session and then unlock before attempting to store the session.
        let (session, store_it) = {
            let store_ug = this.session.store.inner.upgradable_read();
            if let Some(sess) = store_ug.get(&this.session.id.0.to_string()) {
                let session = {
                    let inner = sess.lock();
                    inner.clone()
                };
                (Some(session), true)
            } else {
                (None, false)
            }
        };

        if store_it {
            let _ = block_on(this.session.store.store_session(session.unwrap())).unwrap();
        }

        println!("Finished session store");
        Poll::Ready(Ok(future))
    }
}
