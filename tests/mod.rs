#[cfg(test)]

extern crate raft;

use raft::*;
use std::time::{Instant, Duration};
use futures::executor::block_on;

#[test]
fn test_intialize_follower() {
    let node = Node::new(0);
    assert_eq!(State::Follower, node.state)
}

#[test]
fn test_follower_tick() {
    let mut follower = Node::new(0);
    let mut instant = Instant::now();
    instant = instant + Duration::from_millis(100);
    follower.tick(instant);
    assert_eq!(State::Follower, follower.state);
}

#[test]
fn test_candidate() {
    let mut candidate = Node::new(0);
    let mut instant = Instant::now();
    instant = instant + Duration::from_millis(300);
    candidate.tick(instant);
    assert_eq!(State::Candidate, candidate.state);
}

#[test]
fn test_heartbeat() {
    let mut follower = Node::new(0);
    follower.tick(Instant::now() + Duration::from_millis(100));
    block_on(follower.append_entry(0, Instant::now() + Duration::from_millis(100)));
    follower.tick(Instant::now() + Duration::from_millis(200));
    block_on(follower.append_entry(0, Instant::now() + Duration::from_millis(300)));
    follower.tick(Instant::now() + Duration::from_millis(350));

    assert_eq!(State::Follower, follower.state)
}

#[test]
fn test_reject_leader() {
    let mut follower = Node::new(0);
    follower.tick(Instant::now() + Duration::from_millis(100));
    block_on(follower.append_entry(2, Instant::now() + Duration::from_millis(100)));
    follower.tick(Instant::now() + Duration::from_millis(200));
    block_on(follower.append_entry(1, Instant::now() + Duration::from_millis(300)));
    follower.tick(Instant::now() + Duration::from_millis(400));

    assert_eq!(State::Candidate, follower.state)        
}

#[test]
fn test_request_vote_return_true() {
    let mut follower = Node::new(0);
    let candidate_uuid = 1;
    let mandate = 2;
    assert!(block_on(follower.request_vote(candidate_uuid, mandate, Instant::now())))
}

#[test]
fn test_request_vote_return_false() {
    let mut follower = Node::new(0);
    let candidate_a_uuid = 1;
    let candidate_b_uuid = 2;
    let mandate = 2;
    block_on(follower.request_vote(candidate_a_uuid, mandate, Instant::now()));
    assert!(!block_on(follower.request_vote(candidate_b_uuid, mandate, Instant::now())))
}

#[test]
fn test_request_vote_reject_old_mandate() {
    let mut follower = Node::new(0);
    let candidate_a_uuid = 1;
    let candidate_b_uuid = 2;
    let mandate = 2;
    block_on(follower.request_vote(candidate_a_uuid, mandate, Instant::now()));
    assert!(!block_on(follower.request_vote(candidate_b_uuid, 1, Instant::now())))
}

#[test]
fn test_leader_iterate_cluster() {
    let mut leader = Node::leader(1);
    let mut follower_a = Node::new(2);
    let mut follower_b = Node::new(3);

    follower_a.tick(Instant::now()+Duration::from_millis(100));
    follower_b.tick(Instant::now()+Duration::from_millis(100));

    let cluster = vec![&mut follower_a, &mut follower_b];
    block_on(leader.iterate(cluster, Instant::now()+Duration::from_millis(150)));

    follower_a.tick(Instant::now()+Duration::from_millis(300));
    follower_b.tick(Instant::now()+Duration::from_millis(300));

    assert_eq!(State::Leader, leader.state);
    assert_eq!(State::Follower, follower_a.state);
    assert_eq!(State::Follower, follower_b.state);
}

#[test]
fn test_candidate_iterate_cluster() {
    let mut candidate = Node::candidate(1);
    let mut follower_a = Node::new(2);
    let mut follower_b = Node::new(3);

    follower_a.tick(Instant::now()+Duration::from_millis(100));
    follower_b.tick(Instant::now()+Duration::from_millis(100));

    let cluster = vec![&mut follower_a, &mut follower_b];
    block_on(candidate.iterate(cluster, Instant::now()+Duration::from_millis(150)));

    follower_a.tick(Instant::now()+Duration::from_millis(300));
    follower_b.tick(Instant::now()+Duration::from_millis(300));

    assert_eq!(State::Leader, candidate.state);
    assert_eq!(State::Follower, follower_a.state);
    assert_eq!(State::Follower, follower_b.state);
}