use std::borrow::ToOwned;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashMap;
use rand::{thread_rng, Rng};
use serde::{Serialize, Serializer};
use serde::de::{Deserialize, Deserializer, SeqAccess, Visitor};
use serde::ser::SerializeSeq;

#[derive(Clone, PartialEq, Debug)]
pub struct Chain {
    map: HashMap<Vec<Option<String>>, HashMap<Option<String>, usize>>,
}

impl Serialize for Chain {

    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let map = &self.map;
        let mut seq = serializer.serialize_seq(Some(map.len()))?;

        for (k, v) in map {
            let element = (
                k.clone(),
                v.iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect::<Vec<(Option<String>, usize)>>()
            );
            seq.serialize_element(&element)?;
        }

        seq.end()
    }
}

struct ChainVisitor;

type VecValue = (Vec<Option<String>>, Vec<(Option<String>, usize)>);

impl<'de> Visitor<'de> for ChainVisitor {

    type Value = Vec<VecValue>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a valid serailized chain")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut vec: Self::Value = Vec::new();
        loop {
            match seq.next_element::<VecValue>()? {
                Some(value) => vec.push(value),
                None => break,
            }
        }
        Ok(vec)
    }
}

impl<'de> Deserialize<'de> for Chain {

    fn deserialize<D>(deserializer: D) -> Result<Chain, D::Error>
    where
        D: Deserializer<'de>,
    {
        let chain = Chain {
            map: deserializer.deserialize_seq(ChainVisitor)?
                .iter()
                .map(|(k, v)| {
                    let inner_map: HashMap<Option<String>, usize> = v.iter().map(|x| x.clone()).collect();
                    (k.clone(), inner_map)
                })
                .collect()
        };
        Ok(chain)
    }
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
        serde_json::to_string(&self)
    }

    pub fn load(data: &str) -> serde_json::Result<Chain> {
        serde_json::from_str(&data)
    }

    pub fn feed(&mut self, string: &str) -> &mut Chain {
        let tokens: Vec<String> = string.split(' ').map(|s| s.to_owned()).collect();

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

    fn generate_from_token(&self, token: String) -> Vec<String> {
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
