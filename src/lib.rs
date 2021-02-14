extern crate rand;
use futures::*;
use std::time::{Instant, Duration};
use rand::{thread_rng, Rng};
use futures::stream::{StreamExt};

#[derive(Debug, PartialEq)]
pub enum State {
    Follower,
    Candidate,
    Leader
}

#[derive(Debug)]
pub struct Node {
    uuid: usize,
    leader_uuid: usize,
    pub state: State,
    last_heartbeat: Instant,
    timeout_heartbeat: Duration,
    term: i64,
}

impl Node {
    pub fn leader(uuid: usize) -> Self {
        let mut rander = thread_rng();

        Node{
            uuid: uuid,
            state: State::Leader,
            last_heartbeat: Instant::now(),
            timeout_heartbeat: Duration::from_millis(rander.gen_range(150..300)),
            term: 0,
            leader_uuid: 0,
        }
    }

    pub fn candidate(uuid: usize) -> Self {
        let mut rander = thread_rng();

        Node{
            uuid: uuid,
            state: State::Candidate,
            last_heartbeat: Instant::now(),
            timeout_heartbeat: Duration::from_millis(rander.gen_range(150..300)),
            term: 1,
            leader_uuid: 0,
        }
    }

    pub fn new(uuid: usize) -> Self {
        let mut rander = thread_rng();

        Node{
            uuid: uuid,
            state: State::Follower,
            last_heartbeat: Instant::now(),
            timeout_heartbeat: Duration::from_millis(rander.gen_range(150..300)),
            term: 0,
            leader_uuid: 0,
        }
    }

    pub fn tick(&mut self, instant: Instant) {
        if self.last_heartbeat + self.timeout_heartbeat <= instant {
            self.state = State::Candidate
        }
    }
    
    pub async fn iterate(&mut self, cluster: Vec<&mut Node>, instant: Instant) {
        if self.state == State::Leader {
            stream::iter(cluster)
                .for_each(move |node| { node.append_entry(self.term, instant) })
                .await;
            return;
        }

        if self.state == State::Candidate {
            let quorum = (cluster.len() as f32*0.5).ceil() as usize;
            let votes = stream::iter(cluster)
                    .map(|node| node.request_vote(self.uuid.clone(), self.term.clone(), instant))
                    .fold(0, |acc, x| async move { if x.await { return acc + 1;} return acc;})
                    .await;
                
            if votes >= quorum {
                self.state = State::Leader;
            }
        }
    }

    pub async fn append_entry(&mut self, term: i64, instant: Instant) {
        if self.term > term {
            return
        }

        self.term = term;
        self.last_heartbeat = instant;
        if self.state != State::Candidate {
            self.state = State::Follower;
        }
    }

    pub async fn request_vote(&mut self, candidate_uuid:usize, mandate: i64, instant: Instant) -> bool {
        if self.term >= mandate {
            return false
        }

        self.leader_uuid = candidate_uuid;
        self.term = mandate;
        self.last_heartbeat = instant;
        return true
    }
}