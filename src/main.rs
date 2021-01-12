use teloxide::prelude::*;
use dotenv::dotenv;
use markov::Chain;
use std::io::ErrorKind;
use std::path::Path;
use std::fs::create_dir_all;
use rand::prelude::*;
use anyhow::Result;
use json;
#[macro_use] extern crate log;

fn get_chain(chat_id: i64) -> Result<Chain<String>> {
    let chain_path = format!("./.versor/markov/chains/{}.chain", chat_id);
    
    match Chain::load(chain_path.clone()) {
        Ok(chain) => Ok(chain),
        Err(err) => {
            if err.kind() == ErrorKind::NotFound {
                trace!("Creating new chain for chain {}", chat_id);
                Ok(Chain::new())
            } else {
                error!("Couldn't load chain file for chain {}: {}", chat_id, chain_path);
                Err(err.into())
            }
        }
    }
}

async fn save_chain(chain: Chain<String>, chat_id: i64) -> Result<()> {
    let chain_path = format!("./.versor/markov/chains/{}.chain", chat_id);
    let dir_path = Path::new(&chain_path).parent().expect("Impossible");

    if !dir_path.exists() {
        if let Err(e) = create_dir_all(dir_path) {
            error!("Couldn't create chain storage directory {}: {:?}", dir_path.display(), e);
            return Err(e.into());
        }
    }

    chain.save(chain_path.clone())?;
    Ok(())
}

async fn train(msg: &UpdateWithCx<Message>) -> Result<()> {
    if let Some(doc) = msg.update.document() {
        let bot = Bot::from_env();
        let filepath = bot.get_file(doc.file_id.clone()).send().await?.file_path;
        let url = format!("https://api.telegram.org/file/bot{}/{}", msg.bot.token(), filepath);
        let req = reqwest::get(&url).await?;
        let contents = req.text().await?;
        let data = json::parse(&contents)?;
        let messages = &data["messages"];
        if let json::JsonValue::Array(array) = messages {
            let mut chain = get_chain(msg.chat_id())?;

            for message in array.iter() {
                if message["type"] == "message" {
                    chain.feed_str(message["text"].as_str().unwrap_or_default());
                }
            }

            save_chain(chain, msg.chat_id()).await?;

            msg.reply_to("Обучился").send().await?;
        }
    }
    Ok(())
}

async fn talk(msg: &UpdateWithCx<Message>) -> Result<()> {
    let divisor: u32 = 20;
    let id = msg.bot.get_me().send().await?.user.id;

    if Some(id) == msg.update.reply_to_message().map(Message::from).flatten().map(|u| u.id)
       || (thread_rng().gen_ratio(1, divisor)) {
       let mut chain = get_chain(msg.chat_id())?;

       chain.feed_str(msg.update.text().unwrap_or_default());

       let mut tokens: Vec<&str> = msg.update.text().unwrap_or_default().split(" ").collect();

       tokens.shuffle(&mut thread_rng());
       for token in tokens {
           let reply = chain.generate_str_from_token(token);
           if reply.len() > 0 {
               msg.reply_to(chain.generate_str()).send().await?;
               break;
           }
       }

       save_chain(chain, msg.chat_id()).await?;
    }

    Ok(())
    

}

async fn run() {
    teloxide::enable_logging!();

    if let Err(err) = dotenv() {
        if !err.not_found() {
            error!("Failed to parse .env file");
        }
    }

    log::info!("Starting dices_bot...");

    let bot = Bot::from_env();

    teloxide::repl(bot, |message| async move {
        talk(&message).await;
        train(&message).await;
        ResponseResult::<()>::Ok(())
    })
    .await;
}

#[tokio::main]
async fn main() {
    run().await;
}
