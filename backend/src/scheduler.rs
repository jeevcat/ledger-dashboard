use std::{sync::Arc, time::Duration};

use actix::prelude::*;
use log::info;

use crate::n26::N26;

// Define actor
pub struct Scheduler {
    n26: Arc<N26>,
}

// Provide Actor implementation for our actor
impl Actor for Scheduler {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        info!("Scheduler started.");

        // Disable scheduling for now
        // self.schedule_task(ctx);
    }

    fn stopped(&mut self, _ctx: &mut Context<Self>) {
        info!("Scheduler stopped.");
    }
}

impl Scheduler {
    pub fn start(n26: Arc<N26>) {
        Scheduler { n26 }.start();
    }

    fn schedule_task(&self, ctx: &mut Context<Self>) {
        ctx.spawn(execute_task(self.n26.clone()).actfuture());

        ctx.run_later(duration_until_next(), move |this, ctx| {
            this.schedule_task(ctx)
        });
    }
}

fn duration_until_next() -> Duration {
    Duration::from_secs(60)
}

async fn execute_task(n26: Arc<N26>) {
    // Make sure we occasionally fetch new data from N26
    n26.invalidate_cache();

    let success = n26.attempt_refresh_authentication().await;
    info!(
        "Scheduled token check: {}",
        if success { "valid" } else { "failed" }
    );
}
