extern crate rand;
use rand::Rng;

#[macro_use]
extern crate serde_derive;

extern crate chrono;

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

extern crate crypto;
use crypto::digest::Digest;
use crypto::sha2::Sha256;

#[derive(Serialize, Debug)]
pub struct Block {
    data: String,
    timestamp: i64,
    block_hash: String,
    parent_block_hash: String,
    validator_address: String,
}

impl Block {
    pub fn new(input: String, parent_hash: String, validator: String) -> Block {
        let mut block = Block {
            data: input,
            timestamp: chrono::Utc::now().timestamp_millis(),
            block_hash: String::from(""),
            parent_block_hash: parent_hash,
            validator_address: validator,
        };
        block.generate_hash();
        block
    }

    fn generate_hash(&mut self) {
        let mut hasher = Sha256::new();
        hasher.input(self.data.as_bytes());
        hasher.input(self.timestamp.to_string().as_bytes());
        hasher.input(self.parent_block_hash.as_bytes());
        hasher.input(self.validator_address.as_bytes());
        self.block_hash = hasher.result_str();
    }
}

#[derive(Serialize, Debug)]
pub struct Chain {
    chain: Vec<Block>,
}

impl Chain {
    pub fn new() -> Chain {
        let chain = Chain { chain: Vec::new() };
        chain
    }

    pub fn new_block(&mut self, block: Block) {}
}

pub fn select_winner(validators: &BTreeMap<String, u32>) -> String {
    let mut rng = rand::thread_rng();

    let max: u32 = validators.values().sum();
    let winning_ticket: u32 = rng.gen_range(0, max);

    let mut last_value: u32 = 0;
    let mut winning_key = String::from("");
    for (key, &value) in validators {
        if winning_ticket > last_value && winning_ticket <= last_value + value {
            winning_key = key.clone();
        }
        last_value = value;
    }

    winning_key
}

fn main() {
    let chain = Arc::new(Mutex::new(Chain::new()));
    {
        let root = Block::new(
            String::from("Genesis"),
            (0..64).map(|_| "0").collect(),
            (0..64).map(|_| "0").collect(),
        );

        let chain_temp = chain.clone();
        chain_temp.lock().unwrap().chain.push(root);
        println!("{:#?}", chain_temp.lock().unwrap().chain);
    }

    let validators = Arc::new(Mutex::new(BTreeMap::new()));
    {
        let validators_temp = validators.clone();
        validators_temp
            .lock()
            .unwrap()
            .insert(String::from("000000000000"), 100000);
        validators_temp
            .lock()
            .unwrap()
            .insert(String::from("000000000001"), 50000);
    }

    let chain_validators = validators.clone();
    let chain_append = thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(30));

        let winner = select_winner(&chain_validators.lock().unwrap());
        println!("{}", winner);
    });

    chain_append.join().unwrap();
}
