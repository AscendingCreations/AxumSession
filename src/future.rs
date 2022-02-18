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
    pub struct AxumDatabaseResponseFuture<F> {
        #[pin]
        pub(crate) future: F,
        pub(crate) session: AxumSession,
    }
}
/*
/// This Portion runs when the Route has finished running.
/// It can not See any Extensions for some reason...
impl<F, ResBody, E> Future for AxumDatabaseResponseFuture<F>
where
    F: Future<Output = Result<Response<ResBody>, E>>,
{
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();
        let future = ready!(this.future.poll(cx))?;
        //Lets lock get a clone of the session and then unlock before attempting to store the session.
        let session_data = {
            let store_ug = this.session.store.inner.upgradable_read();
            if let Some(sess) = store_ug.get(&this.session.id.0.to_string()) {
                Some({
                    let inner = sess.lock();
                    inner.clone()
                })
            } else {
                None
            }
        };

        //Any better way to do this as i could only figure this out? nvm this is broken too since it doesnt insert into the database.
        //if this.store_future.is_none() {
        let sess = this.session.clone();

        let fut = Box::pin(async move {
            if let Some(data) = session_data {
                sess.store.store_session(data).await.unwrap();
            }
        });

        let _ = fut.as_mut().poll(cx);

        println!("Finished session store");
        Poll::Ready(Ok(future))
    }
}*/
