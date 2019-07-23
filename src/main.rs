//! # A Raft Sandbox
//!
//! This project will simulate a distributed Raft system using local threads
use raft::prelude::*;
use raft::storage::MemStorage;
use std::{thread, time};
use std::sync::mpsc;

/// Stores the amount of nodes we want to simulate
const NODE_COUNT: u32 = 1;

enum Msg {
    Raft(Message),
}

fn main() {
    // We want to create a pool of mailboxes to recieve and transmit information from all the
    // nodes.
    let (mut tx_pool, mut rx_pool) = (Vec::new(), Vec::new());
    for i in 0..NODE_COUNT {
        let (tx, rx) = mpsc::channel();
        tx_pool.push(tx);
        rx_pool.push(rx);
    }
    //And now we'll create a pool of handles to store the handles of each node
    let mut handle_pool = Vec::new(); 

    // We'll create each node here. We'll use the rx pool as the iterator since we need each one
    for (i, rx) in rx_pool.into_iter().enumerate() {
        let node_config = Config {
            id: (i + 1) as u64,
            ..Default::default()
        };
        // And now we'll clone the mailboxes for each node
        let mailboxes = tx_pool.clone();
        // And we will create the node
        node_config.validate().unwrap(); // validate the node's config
        let storage = MemStorage::new(); // The node's storage
        let mut node = RawNode::new(&node_config, storage, vec![]).unwrap();
        // We'll spawn a thread for this node
        let handle = thread::spawn(move || {
            thread::sleep(time::Duration::from_millis(10));
            // We'll have the node wait for a message
            loop {
                match rx.try_recv() {
                    Ok(msg) => node.step(msg).unwrap(),
                    Err(mpsc::TryRecvError::Empty) => break,
                    Err(mpsc::TryRecvError::Disconnected) => return,
                }
            }

            // If we get to this point, this means that the nodes are not initialized
        });
        // We push the handle for this thread so we can access it later
        handle_pool.push(handle);
    }
    //And we wait for all threads to finish
    for thread in handle_pool {
        thread.join().unwrap();
    }
}


// We must see if any message recieved is a message which could be used to initialize a node.
// These include a RequestVote message, a Request Prevote message or the first heartbeat where the
// commit number is 0
fn is_initial_msg(msg: &Message) -> bool {
    let msg_type = msg.get_msg_type();
    msg_type == MessageType::MsgRequestVote
        || msg_type == MessageType::MsgRequestPreVote
        || (msg_type == MessageType::MsgHeartbeat && msg.commit == 0)
}


