extern crate rand;
use rand::Rng;

#[macro_use]
extern crate serde_derive;

extern crate chrono;

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use std::error;
use std::fmt;

extern crate crypto;
use crypto::digest::Digest;
use crypto::sha2::Sha256;

#[derive(Serialize, Debug, Clone)]
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

    fn blank_hash() -> String {
        Block::new(String::from(""), String::from(""), String::from("")).block_hash
    }
}

#[derive(Debug)]
pub enum BlockchainError {
    UnknownValidator,
}

impl fmt::Display for BlockchainError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            BlockchainError::UnknownValidator => write!(f, "could not suggest block, address has no stake"),
        }
    }
}

impl error::Error for BlockchainError {
    fn description(&self) -> &str {
        match *self {
            BlockchainError::UnknownValidator => "could not mine block, hit iteration limit",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        None
    }
}

#[derive(Debug)]
pub struct Chain {
    chain: Vec<Block>,
    candidates: Vec<Block>,
    validators: BTreeMap<String, u32>,
    stake: BTreeMap<String, u32>,
}

impl Chain {
    pub fn new() -> Chain {
        let chain = Chain { 
            chain: Vec::new(),
            candidates: Vec::new(),
            validators: BTreeMap::new(),
            stake: BTreeMap::new(),
         };
        chain
    }

    pub fn suggest_block(&mut self, data: String, validator: String) -> Result<(),BlockchainError> {
        let stake = self.stake.get(&validator);
        if stake == None {
            return Err(BlockchainError::UnknownValidator);
        }

        let blank_hash = Block::blank_hash();
        let prev_hash = match self.chain.last(){
            Some(b) => &b.block_hash,
            None => &blank_hash,
        };

        self.candidates.push(Block::new(data, prev_hash.to_string(), validator.clone()));
        self.validators.insert(validator.clone(), *stake.unwrap());

        Ok(())
    }

    pub fn select_winner(&mut self) {
        let mut rng = rand::thread_rng();

        let max: u32 = self.validators.values().sum();
        let winning_ticket: u32 = rng.gen_range(0, max);

        let mut last_value: u32 = 0;
        let mut winning_key = String::from("");
        for (key, value) in &self.validators {
            if winning_ticket > last_value && winning_ticket <= last_value + value {
                winning_key = key.clone();
            }
            last_value = *value;
        }

        for block in &self.candidates {
            if block.validator_address == winning_key {
                self.chain.push((*block).clone());
                break;
            }
        }
        self.validators.clear();
        self.candidates.clear();
    }
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

    {
        let chain_temp = chain.clone();
        chain_temp
            .lock()
            .unwrap()
            .stake
            .insert(String::from("000000000000"), 100000);
        chain_temp
            .lock()
            .unwrap()
            .stake
            .insert(String::from("000000000001"), 50000);
    }

    {
        chain.lock().unwrap().suggest_block(String::from("Wallet A"), String::from("000000000000"));
        chain.lock().unwrap().suggest_block(String::from("Wallet B"), String::from("000000000001"));
    }

    let append_chain = chain.clone();
    let append_thread = thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(30));

        append_chain.lock().unwrap().select_winner();
        println!("{:#?}", append_chain.lock().unwrap().chain);
    });

    append_thread.join().unwrap();
}
