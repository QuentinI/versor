use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::fs;

use crate::chain::Chain;
use crate::settings::Settings;

type ArcMutex<T> = Arc<Mutex<T>>;

struct CounterChain {
    counter: u8,
    chain: ArcMutex<Chain>,
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
    cache: ArcMutex<HashMap<i64, CounterChain>>,
    cycle: u8,
}

impl ChainCache {

    pub fn new(settings: &Settings) -> ChainCache {
        ChainCache {
            cache: Arc::new(Mutex::new(HashMap::new())),
            cycle: settings.constants.cache_cycle,
        }
    }

    fn file_name(chat_id: i64) -> String {
        String::from(format!("./.chains/{}.json", chat_id))
    }

    fn get_file(&self, chat_id: i64) -> Option<String> {
        fs::read_to_string(ChainCache::file_name(chat_id)).ok()
    }

    fn save_file(&self, chat_id: i64, chain: &Chain) -> Result<()> {
        let file_name = ChainCache::file_name(chat_id);
        info!("Saving chain to a file for {} into {}", chat_id, file_name);

        let dir_path = std::path::Path::new(&file_name).parent().expect("Impossible");

        if !dir_path.exists() {
            if let Err(e) = fs::create_dir_all(dir_path) {
                error!("Couldn't create chain storage directory {}: {:?}", dir_path.display(), e);
                return Err(e.into());
            }
        }

        fs::write(ChainCache::file_name(chat_id), chain.save()?)?;
        Ok(())
    }

    pub fn get_chain<'a>(&'a self, chat_id: i64) -> Result<ArcMutex<Chain>> {
        let mut cache = self.cache.lock().unwrap();

        if let Some(counter_chain) = cache.get_mut(&chat_id) {
            return Ok(counter_chain.chain.clone())
        };

        let chain = match self.get_file(chat_id) {
            Some(data) => match Chain::load(&data) {
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
                self.save_file(chat_id, &chain)?;
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
                    self.save_file(chat_id, &chain)
                } else {
                    counter_chain.counter -= 1;
                    Ok(())
                }
            }
            None => anyhow::bail!("Save chain is called before get chain")
        }
    }
}
