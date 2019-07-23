//! # A Raft Sandbox
//!
//! This project will simulate a distributed Raft system using local threads
use raft::prelude::*;
use raft::storage::MemStorage;
use std::{thread, time};
use std::sync::mpsc;

/// Stores the amount of nodes we want to simulate
const NODE_COUNT: u32 = 1;

fn main() {
    // We want to create a pool of mailboxes to recieve and transmit information from all the
    // nodes.
    let (mut tx_pool, mut rx_pool) = (Vec::new(), Vec::new());
    for _ in 0..NODE_COUNT {
        let (tx, rx) = mpsc::channel();
        tx_pool.push(tx);
        rx_pool.push(rx);
    }
    //And now we'll create a pool of handles to store the handles of each node
    //let mut handle_pool = Vec::new(); 

    // We'll create each node here. We'll remember that each mailbox is tied to its 
    for i in 0..NODE_COUNT {
        let node_config = Config {
            id: (i + 1) as u64,
            ..Default::default()
        };
        // We'll first create the node
        // We start with the storage
        let storage = MemStorage::new();
        //Then we make sure the configuration is valid
        node_config.validate().unwrap();
        //And now we'll just use the MemStorage as our new node's storage and we'll create our own
        //node
        let mut node = RawNode::new(&node_config, storage, vec![]).unwrap();
        node.raft.become_candidate();
        node.raft.become_leader();
        let timeout = time::Duration::from_millis(100);
        let mut remaining_timeout = timeout;
        // And now we'll tick it
        loop {
            let now = time::Instant::now();


            match rx_pool[i as usize].recv_timeout(remaining_timeout) {
                Ok(()) => {
                    unimplemented!();
                },
                Err(mpsc::RecvTimeoutError::Timeout) => (),
                Err(mpsc::RecvTimeoutError::Disconnected) => unimplemented!(),
            }

            let elapsed = now.elapsed();
            if elapsed >= remaining_timeout {
                remaining_timeout = timeout;
                // We'll drive raft each 100ms interval
                node.tick();
            } else {
                remaining_timeout -= elapsed;
            }

            
        }
    }
}
