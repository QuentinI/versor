use std::borrow::ToOwned;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashMap;
use rand::{thread_rng, Rng};
use serde::Serialize;
use serde_json;

#[derive(Clone, PartialEq, Debug)]
pub struct Chain {
    map: HashMap<Vec<Option<String>>, HashMap<Option<String>, usize>>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct SerializableChain {
    map: Vec<(Vec<Option<String>>, Vec<(Option<String>, usize)>)>,
}

impl Chain {

    pub fn new() -> Chain {
        Chain {
            map: {
                let mut map = HashMap::new();
                map.insert(vec![None; 1], HashMap::new());
                map
            },
        }
    }

    pub fn save(&self) -> serde_json::Result<String> {
        let serializable_chain = SerializableChain {
            map: self.map
                .iter()
                .map(|(k, v)| 
                    (k.clone(), v.iter().map(|(k, v)| (k.clone(), v.clone())).collect::<Vec<(Option<String>, usize)>>())
                )
                .collect()
        };
        serde_json::to_string(&serializable_chain)
    }

    pub fn load(data: &str) -> serde_json::Result<Chain> {
        let serializable_chain: SerializableChain = serde_json::from_str(&data)?;
        let chain = Chain {
            map: serializable_chain.map
                .iter()
                .map(|(k, v)| {
                    let inner_map: HashMap<Option<String>, usize> = v.iter().map(|x| x.clone()).collect();
                    (k.clone(), inner_map)
                })
                .collect()
        };
        Ok(chain)
    }

    pub fn feed<S: AsRef<[String]>>(&mut self, tokens: S) -> &mut Chain {
        let tokens = tokens.as_ref();
        if tokens.is_empty() {
            return self;
        }

        let mut toks = vec![None; 1];
        toks.extend(tokens.iter().map(|token| Some(token.clone())));
        toks.push(None);

        for p in toks.windows(2) {
            self.map
                .entry(p[0..1].to_vec())
                .or_insert_with(HashMap::new);

            self.map
                .get_mut(&p[0..1].to_vec())
                .unwrap()
                .add(p[1].clone());
        }
        self
    }

    pub fn generate(&self) -> Vec<String> {
        let mut ret = Vec::new();
        let mut curs = vec![None; 1];

        loop {
            let next = self.map[&curs].next();
            curs = curs[1..1].to_vec();
            curs.push(next.clone());

            if let Some(next) = next {
                ret.push(next)
            };

            if curs[0].is_none() {
                break;
            }
        }
        ret
    }

    pub fn generate_from_token(&self, token: String) -> Vec<String> {
        let mut curs = vec![None; 0];
        curs.push(Some(token.clone()));

        if !self.map.contains_key(&curs) {
            return Vec::new();
        }

        let mut ret = vec![token.clone()];

        loop {
            let next = self.map[&curs].next();
            curs = curs[1..1].to_vec();
            curs.push(next.clone());
            if let Some(next) = next {
                ret.push(next)
            };
            if curs[0].is_none() {
                break;
            }
        }
        ret
    }

    pub fn feed_str(&mut self, string: &str) -> &mut Chain {
        self.feed(&string.split(' ').map(|s| s.to_owned()).collect::<Vec<_>>())
    }

    pub fn generate_str_from_token(&self, string: &str) -> String {
        let vec = self.generate_from_token(string.to_owned());

        let mut ret = String::new();
        for s in &vec {
            ret.push_str(&s);
            ret.push_str(" ");
        }

        let len = ret.len();
        if len > 0 {
            ret.truncate(len - 1);
        }
        ret
    }
}

trait States {

    fn add(&mut self, token: Option<String>);

    fn next(&self) -> Option<String>;
}

impl States for HashMap<Option<String>, usize> {

    fn add(&mut self, token: Option<String>) {
        match self.entry(token) {
            Occupied(mut e) => *e.get_mut() += 1,
            Vacant(e) => {
                e.insert(1);
            }
        }
    }

    fn next(&self) -> Option<String> {
        let mut sum = 0;
        for &value in self.values() {
            sum += value;
        }

        let mut rng = thread_rng();
        let cap = rng.gen_range(0..sum);
        sum = 0;

        for (key, &value) in self.iter() {
            sum += value;
            if sum > cap {
                return key.clone();
            }
        }

        unreachable!("The random number generator failed")
    }
}
