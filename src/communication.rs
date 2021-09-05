use anyhow::Result;
use json;
use rand::prelude::*;
use std::sync::Arc;
use teloxide::{dispatching::*, prelude::*};

use crate::cache::ChainCache;
use crate::settings::Settings;

pub async fn process_message(
    bot: Bot,
    msg: &UpdateWithCx<Bot, Message>,
    settings: Arc<Settings>,
    chain_cache: ChainCache,
) -> Result<()> {
    let (first, second) = tokio::join!(
        train(bot.clone(), msg, settings.clone(), chain_cache.clone()),
        talk(bot, msg, settings, chain_cache),
    );
    first?;
    second?;
    Ok(())
}

async fn train(
    bot: Bot,
    msg: &UpdateWithCx<Bot, Message>,
    _: Arc<Settings>,
    chain_cache: ChainCache,
) -> Result<()> {
    if let Some(doc) = msg.update.document() {
        info!("Starting training");
        let filepath = bot.get_file(doc.file_id.clone()).send().await?.file_path;

        let url = format!(
            "https://api.telegram.org/file/bot{}/{}",
            msg.requester.token(),
            filepath,
        );

        let req = reqwest::get(&url).await?;
        let contents = req.text().await?;

        info!("Parsing document");
        let data = json::parse(&contents)?;
        let messages = &data["messages"];
        info!("Parsed");

        if let json::JsonValue::Array(array) = messages {
            info!("Learning messages");
            {
                let chain = chain_cache.get_chain(msg.chat_id())?;
                let mut chain = chain.lock().unwrap();
    
                info!("Learning from {} messages", array.len());
                for message in array.iter() {
                    if message["type"] == "message" {
                        chain.feed_str(message["text"].as_str().unwrap_or_default());
                    }
                }
    
                info!("Saving chain");
                chain_cache.save_chain(msg.chat_id())?;
            }

            msg.reply_to("Trained 💪").send().await?;
        } else {
            error!("No messages");
        }
    }
    Ok(())
}

async fn talk(
    _: Bot,
    msg: &UpdateWithCx<Bot, Message>,
    settings: Arc<Settings>,
    chain_cache: ChainCache,
) -> Result<()> {
    let id = msg.requester.get_me().send().await?.user.id;

    if Some(id)
        == msg
            .update
            .reply_to_message()
            .map(Message::from)
            .flatten()
            .map(|u| u.id)
        || (thread_rng().gen_ratio(1, settings.constants.divisor.clone()))
    {
        info!("Replying");
        let reply: Option<String>;
        {
            let chain = chain_cache.get_chain(msg.chat_id())?;
            let mut chain = chain.lock().unwrap();
            chain.feed_str(msg.update.text().unwrap_or_default());

            let mut tokens: Vec<&str> = msg.update.text().unwrap_or_default().split(" ").collect();

            tokens.shuffle(&mut thread_rng());
            reply = tokens.iter()
                .map(|token| chain.generate_str_from_token(token))
                .filter(|reply| reply.len() > 0)
                .next();
        }
        
        if let Some(reply) = reply {
            msg.reply_to(reply).send().await?;
        }

        chain_cache.save_chain(msg.chat_id())?;
    }

    Ok(())
}
