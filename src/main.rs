#[cfg(feature = "json")]
extern crate serde_json;
#[macro_use]
extern crate log;
extern crate rand;
extern crate serde;

pub mod cache;
pub mod chain;
pub mod communication;
pub mod settings;

use std::sync::Arc;
use teloxide::{dispatching::*, prelude::*};
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::cache::ChainCache;
use crate::communication::process_message;
use crate::settings::Settings;

async fn run() {
    dotenv::dotenv().ok();
    let settings = match Settings::new() {
        Ok(s) => Arc::new(s),
        Err(r) => panic!("{}", r),
    };

    teloxide::enable_logging_with_filter!(log::LevelFilter::Debug);
    log::info!("Starting bot...");

    let bot = teloxide::Bot::new(settings.telegram.token.clone());
    let chain_cache = ChainCache::new(&settings);
    let handler = Arc::new(process_message);
    let cloned_bot = bot.clone();

    Dispatcher::new(bot.clone())
        .messages_handler(move |rx: DispatcherHandlerRx<_, Message>| {
            UnboundedReceiverStream::new(rx).for_each_concurrent(None, move |message| {
                let handler = handler.clone();
                let cloned_bot = cloned_bot.clone();
                let settings = settings.clone();
                let chain_cache = chain_cache.clone();

                async move {
                    handler(cloned_bot, &message, settings, chain_cache)
                        .await
                        .log_on_error()
                        .await;
                }
            })
        })
        .setup_ctrlc_handler()
        .dispatch_with_listener(
            update_listeners::polling_default(bot).await,
            teloxide::error_handlers::LoggingErrorHandler::with_custom_text(
                "An error from the update listener",
            ),
        )
        .await;
}

#[tokio::main]
async fn main() {
    run().await;
}
