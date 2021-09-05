use crate::schema::chat_chains;

#[derive(Queryable)]
pub struct ChatChain {
    pub chat_id: i64,
    pub chain: String,
}

#[derive(Insertable)]
#[table_name = "chat_chains"]
pub struct NewChatChain<'a> {
    pub chat_id: &'a i64,
    pub chain: &'a String,
}

impl NewChatChain<'_> {
    pub fn new<'a>(chat_id: &'a i64, chain: &'a String) -> NewChatChain<'a> {
        NewChatChain {
            chat_id,
            chain,
        }
    }
}
