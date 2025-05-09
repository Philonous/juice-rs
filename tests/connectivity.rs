use std::sync::mpsc::{Receiver, channel};
use std::sync::{Arc, Barrier};
use std::thread::{sleep, spawn};
use std::time::Duration;

use libjuice_rs::{Agent, Handler, State};

include!("../src/test_util.rs");

#[test]
fn connectivity_no_trickle() {
    logger_init();

    let gathering_barrier = Arc::new(Barrier::new(3));

    let (first_tx, first_rx) = channel();
    let first_handler = Handler::default()
        .gathering_done_handler({
            let barrier = gathering_barrier.clone();
            move || {
                log::info!("first agent finished gathering");
                barrier.wait();
            }
        })
        .state_handler(move |state| log::info!("first changed state to: {:?}", state))
        .recv_handler(move |packet| {
            log::debug!("first received {:?}", packet);
            let _ = first_tx.send(packet.to_vec());
        });

    let first = Agent::builder(first_handler).build().unwrap();

    let (second_tx, second_rx) = channel();
    let second_handler = Handler::default()
        .gathering_done_handler({
            let barrier = gathering_barrier.clone();
            move || {
                log::info!("second agent finished gathering");
                barrier.wait();
            }
        })
        .state_handler(move |state| log::info!("second changed state to: {:?}", state))
        .recv_handler(move |packet| {
            log::debug!("second received {:?}", packet);
            let _ = second_tx.send(packet.to_vec());
        });
    let second = Agent::builder(second_handler)
        .with_port_range(5000, 5010)
        .build()
        .unwrap();

    first.gather_candidates().unwrap();
    second.gather_candidates().unwrap();

    gathering_barrier.wait();

    let first_desc = first.get_local_description().unwrap();
    second.set_remote_description(first_desc).unwrap();

    let second_desc = second.get_local_description().unwrap();
    first.set_remote_description(second_desc).unwrap();

    sleep(Duration::from_secs(2));

    assert_eq!(first.get_state(), State::Completed);
    assert_eq!(second.get_state(), State::Completed);

    log::info!(
        "first selected candidates: {:?}",
        first.get_selected_candidates()
    );
    log::info!(
        "second selected candidates: {:?}",
        second.get_selected_candidates()
    );
    log::info!(
        "first selected addresses: {:?}",
        first.get_selected_addresses()
    );
    log::info!(
        "second selected addresses: {:?}",
        second.get_selected_addresses()
    );

    first.send("hello".as_bytes()).unwrap();
    assert_eq!(
        second_rx.recv_timeout(Duration::from_secs(1)),
        Ok("hello".into())
    );

    second.send("world".as_bytes()).unwrap();
    assert_eq!(
        first_rx.recv_timeout(Duration::from_secs(1)),
        Ok("world".into())
    );
}

enum TrickleEvent {
    Candidate(String),
    Eof,
}

// tricky trickle
fn trickle_signaling(ch: Receiver<TrickleEvent>, agent: Arc<Agent>) {
    let mut counter = 0;
    loop {
        match ch.recv_timeout(Duration::from_secs(1)) {
            Ok(TrickleEvent::Candidate(sdp)) => agent.add_remote_candidate(sdp).unwrap(),
            Ok(TrickleEvent::Eof) => {
                agent.set_remote_gathering_done().unwrap();
                break;
            }
            Err(_) => {
                counter += 1;
                if counter == 3 {
                    break;
                }
            }
        }
    }
}

#[test]
fn connectivity_trickle() {
    logger_init();

    let gathering_barrier = Arc::new(Barrier::new(3));

    let (first_tx, first_rx) = channel();
    let (first_candidate_tx, first_candidate_rx) = channel();
    let first_handler = Handler::default()
        .gathering_done_handler({
            let barrier = gathering_barrier.clone();
            let first_candidate_tx = first_candidate_tx.clone();
            move || {
                log::info!("first agent finished gathering");
                let _ = first_candidate_tx.send(TrickleEvent::Eof);
                barrier.wait();
            }
        })
        .state_handler(move |state| log::info!("first changed state to: {:?}", state))
        .recv_handler(move |packet| {
            log::debug!("first received {:?}", packet);
            let _ = first_tx.send(packet.to_vec());
        })
        .candidate_handler({
            move |sdp| {
                log::debug!("first local candidate {:?}", sdp);
                let _ = first_candidate_tx.send(TrickleEvent::Candidate(sdp));
            }
        });

    let bind = "127.0.0.1".parse().unwrap();
    let first = Arc::new(
        Agent::builder(first_handler)
            .with_bind_address(&bind)
            .build()
            .unwrap(),
    );

    let (second_tx, second_rx) = channel();
    let (second_candidate_tx, second_candidate_rx) = channel();
    let second_handler = Handler::default()
        .gathering_done_handler({
            let barrier = gathering_barrier.clone();
            let second_candidate_tx = second_candidate_tx.clone();
            move || {
                log::info!("second agent finished gathering");
                let _ = second_candidate_tx.send(TrickleEvent::Eof);
                barrier.wait();
            }
        })
        .state_handler(move |state| log::info!("second changed state to: {:?}", state))
        .recv_handler(move |packet| {
            log::debug!("second received {:?}", packet);
            let _ = second_tx.send(packet.to_vec());
        })
        .candidate_handler(move |sdp| {
            log::debug!("second local candidate {:?}", sdp);
            let _ = second_candidate_tx.send(TrickleEvent::Candidate(sdp));
        });

    let second = Arc::new(Agent::builder(second_handler).build().unwrap());

    let handle1 = {
        let first = first.clone();
        spawn(move || trickle_signaling(second_candidate_rx, first))
    };

    let handle2 = {
        let second = second.clone();
        spawn(move || trickle_signaling(first_candidate_rx, second))
    };

    // exchange descriptions

    let first_desc = first.get_local_description().unwrap();
    second.set_remote_description(first_desc).unwrap();

    let second_desc = second.get_local_description().unwrap();
    first.set_remote_description(second_desc).unwrap();

    // and then start gathering
    first.gather_candidates().unwrap();
    second.gather_candidates().unwrap();

    gathering_barrier.wait();

    sleep(Duration::from_secs(2));

    assert!(matches!(
        first.get_state(),
        State::Connected | State::Completed
    ));
    assert!(matches!(
        second.get_state(),
        State::Connected | State::Completed
    ));

    log::info!(
        "first selected candidates: {:?}",
        first.get_selected_candidates()
    );
    log::info!(
        "second selected candidates: {:?}",
        second.get_selected_candidates()
    );
    log::info!(
        "first selected addresses: {:?}",
        first.get_selected_addresses()
    );
    log::info!(
        "second selected addresses: {:?}",
        second.get_selected_addresses()
    );

    first.send("hello".as_bytes()).unwrap();
    assert_eq!(
        second_rx.recv_timeout(Duration::from_secs(1)),
        Ok("hello".into())
    );

    second.send("world".as_bytes()).unwrap();
    assert_eq!(
        first_rx.recv_timeout(Duration::from_secs(1)),
        Ok("world".into())
    );

    handle1.join().unwrap();
    handle2.join().unwrap();
}
