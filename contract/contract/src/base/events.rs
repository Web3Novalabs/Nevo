use soroban_sdk::{Address, BytesN, Env, String, Symbol};

pub fn campaign_created(
    env: &Env,
    id: BytesN<32>,
    title: String,
    creator: Address,
    goal: i128,
    deadline: u64,
) {
    let topics = (Symbol::new(env, "campaign_created"), id, creator);
    env.events().publish(topics, (title, goal, deadline));
}
