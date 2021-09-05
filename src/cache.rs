use anyhow::Result;
use diesel::{pg::PgConnection, prelude::*};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::chain::Chain;
use crate::models::{ChatChain, NewChatChain};
use crate::schema::chat_chains::{columns, dsl::chat_chains};
use crate::settings::Settings;

fn establish_connection(url: &str) -> Arc<Mutex<PgConnection>> {
    Arc::new(Mutex::new(
        PgConnection::establish(url).expect(&format!("Error connecting to {}", url)),
    ))
}

struct CounterChain {
    counter: u8,
    chain: Arc<Mutex<Chain>>,
}

impl CounterChain {

    fn new(counter: u8, chain: Chain) -> CounterChain {
        CounterChain {
            counter,
            chain: Arc::new(Mutex::new(chain)),
        }
    }
}

#[derive(Clone)]
pub struct ChainCache {
    cache: Arc<Mutex<HashMap<i64, CounterChain>>>,
    connection: Arc<Mutex<PgConnection>>,
    cycle: u8,
}

impl ChainCache {

    pub fn new(settings: &Settings) -> ChainCache {
        ChainCache {
            cache: Arc::new(Mutex::new(HashMap::new())),
            connection: establish_connection(&settings.database.url),
            cycle: settings.constants.cache_cycle,
        }
    }

    fn get_database(&self, chat_id: i64) -> Option<ChatChain> {
        chat_chains
            .filter(columns::chat_id.eq(chat_id))
            .load::<ChatChain>(&*self.connection.lock().unwrap())
            .expect("Error loading chain")
            .into_iter()
            .next()
    }

    fn save_database(&self, chat_id: i64, chain: &Chain) -> Result<()> {
        info!("Saving chain to a database for {}", chat_id);

        let chain = chain.save()?;

        diesel::insert_into(chat_chains)
            .values(&NewChatChain::new(&chat_id, &chain))
            .get_result::<ChatChain>(&*self.connection.lock().unwrap())
            .expect("Error saving a chain");

        info!("Saved chain to a database for {}", chat_id);

        Ok(())
    }

    fn update_database(&self, chat_id: i64, chain: &Chain) -> Result<()> {
        info!("Updating chain for {}", chat_id);

        let chain = chain.save()?;

        diesel::update(chat_chains.filter(columns::chat_id.eq(chat_id)))
            .set(columns::chain.eq(&chain))
            .get_result::<ChatChain>(&*self.connection.lock().unwrap())
            .expect("Error saving a chain");

        info!("Updated chain for {}", chat_id);

        Ok(())
    }

    pub fn get_chain<'a>(&'a self, chat_id: i64) -> Result<Arc<Mutex<Chain>>> {
        let mut cache = self.cache.lock().unwrap();

        if let Some(counter_chain) = cache.get_mut(&chat_id) {
            return Ok(counter_chain.chain.clone())
        };

        let chain = match self.get_database(chat_id) {
            Some(chain) => match Chain::load(&chain.chain) {
                Ok(chain) => {
                    info!("Found a chain for {}", chat_id);
                    chain
                }
                Err(err) => {
                    error!("Couldn't load chain for {} bc {}", chat_id, err,);
                    return Err(err.into())
                }
            },
            None => {
                info!("Inserting new chain in a database for {}", chat_id);
                let chain = Chain::new();
                self.save_database(chat_id, &chain)?;
                chain
            }

        };

        cache.insert(chat_id, CounterChain::new(self.cycle, chain.clone()));
        std::mem::drop(cache);
        self.get_chain(chat_id)
    }

    pub fn save_chain(&self, chat_id: i64) -> Result<()> {
        let mut cache = self.cache.lock().unwrap();

        match cache.get_mut(&chat_id) {
            Some(counter_chain) => {
                if counter_chain.counter == 0 {
                    counter_chain.counter = self.cycle;
                    let chain = counter_chain.chain.lock().unwrap();
                    self.update_database(chat_id, &chain)
                } else {
                    counter_chain.counter -= 1;
                    Ok(())
                }
            }
            None => anyhow::bail!("Save chain is called before get chain")
        }
    }
}
