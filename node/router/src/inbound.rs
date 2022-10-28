// Copyright (C) 2019-2022 Aleo Systems Inc.
// This file is part of the snarkOS library.

// The snarkOS library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The snarkOS library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the snarkOS library. If not, see <https://www.gnu.org/licenses/>.

use crate::{Peer, Router, RouterRequest, ALEO_MAXIMUM_FORK_DEPTH};
use snarkos_node_executor::{spawn_task, Executor};
use snarkos_node_messages::*;
use snarkvm::prelude::Network;

use core::time::Duration;
use std::{net::SocketAddr, time::SystemTime};

#[async_trait]
pub trait Inbound<N: Network>: Executor {
    /// Handles the receiving of a message from a peer. Upon success, returns `true`.
    #[rustfmt::skip]
    async fn inbound(&self, peer: &Peer<N>, message: Message<N>, router: &Router<N>) -> bool {
        // Retrieve the peer IP.
        let peer_ip = *peer.ip();

        // Process the message.
        trace!("Received '{}' from {peer_ip}", message.name());
        match message {
            Message::BlockRequest(message) => Self::block_request(message).await,
            Message::BlockResponse(message) => Self::block_response(message).await,
            Message::ChallengeRequest(..) | Message::ChallengeResponse(..) => {
                // Peer is not following the protocol.
                warn!("Peer {peer_ip} is not following the protocol");
                false
            }
            Message::Disconnect(message) => {
                debug!("Disconnecting peer {peer_ip} for the following reason: {:?}", message.reason);
                false
            }
            Message::PeerRequest(..) => Self::peer_request(peer_ip, router).await,
            Message::PeerResponse(message) => Self::peer_response(message, router).await,
            Message::Ping(message) => Self::ping(message, peer_ip, peer).await,
            Message::Pong(message) => Self::pong(message, peer_ip, router).await,
            Message::PuzzleRequest(..) => self.puzzle_request(peer_ip, router).await,
            Message::PuzzleResponse(message) => self.puzzle_response(message, peer_ip).await,
            Message::UnconfirmedBlock(message) => self.unconfirmed_block(message, peer_ip, peer, router).await,
            Message::UnconfirmedSolution(message) => self.unconfirmed_solution(message, peer_ip, router).await,
            Message::UnconfirmedTransaction(message) => self.unconfirmed_transaction(message, peer_ip, router).await,
            Message::NewEpochChallenge(_message) => {
                // todo deal with NewEpochChallenge
                true
            },
        }
    }

    async fn block_request(_message: BlockRequest) -> bool {
        // // Ensure the request is within the accepted limits.
        // let number_of_blocks = end_block_height.saturating_sub(start_block_height);
        // if number_of_blocks > Router::<N>::MAXIMUM_BLOCK_REQUEST {
        //     // Route a `Failure` to the ledger.
        //     let failure = format!("Attempted to request {} blocks", number_of_blocks);
        //     if let Err(error) = state.ledger().router().send(LedgerRequest::Failure(peer_ip, failure)).await {
        //         warn!("[Failure] {}", error);
        //     }
        //     continue;
        // }
        // // Retrieve the requested blocks.
        // let blocks = match state.ledger().reader().get_blocks(start_block_height, end_block_height) {
        //     Ok(blocks) => blocks,
        //     Err(error) => {
        //         // Route a `Failure` to the ledger.
        //         if let Err(error) = state.ledger().router().send(LedgerRequest::Failure(peer_ip, format!("{}", error))).await {
        //             warn!("[Failure] {}", error);
        //         }
        //         continue;
        //     }
        // };
        // // Send a `BlockResponse` message for each block to the peer.
        // for block in blocks {
        //     debug!("Sending 'BlockResponse {}' to {}", block.height(), peer_ip);
        //     if let Err(error) = peer.outbound_socket.send(Message::BlockResponse(Data::Object(block))).await {
        //         warn!("[BlockResponse] {}", error);
        //         break;
        //     }
        // }
        true
    }

    async fn block_response(_message: BlockResponse<N>) -> bool {
        // // Perform the deferred non-blocking deserialization of the block.
        // match block.deserialize().await {
        //     Ok(block) => {
        //         // Route the `BlockResponse` to the ledger.
        //         if let Err(error) = state.ledger().router().send(LedgerRequest::BlockResponse(peer_ip, block)).await {
        //             warn!("[BlockResponse] {}", error);
        //         }
        //     },
        //     // Route the `Failure` to the ledger.
        //     Err(error) => if let Err(error) = state.ledger().router().send(LedgerRequest::Failure(peer_ip, format!("{}", error))).await {
        //         warn!("[Failure] {}", error);
        //     }
        // }
        true
    }

    async fn peer_request(peer_ip: SocketAddr, router: &Router<N>) -> bool {
        // Send a `PeerResponse` message.
        if let Err(error) = router.process(RouterRequest::SendPeerResponse(peer_ip)).await {
            warn!("[PeerRequest] {error}");
        }
        true
    }

    async fn peer_response(message: PeerResponse, router: &Router<N>) -> bool {
        // Adds the given peer IPs to the list of candidate peers.
        if let Err(error) = router.process(RouterRequest::ReceivePeerResponse(message.peers)).await {
            warn!("[PeerResponse] {error}");
        }
        true
    }

    async fn ping(message: Ping, peer_ip: SocketAddr, peer: &Peer<N>) -> bool {
        // Ensure the message protocol version is not outdated.
        if message.version < Message::<N>::VERSION {
            warn!("Dropping {peer_ip} on version {} (outdated)", message.version);
            return false;
        }
        // Ensure the maximum fork depth is correct.
        if message.fork_depth != ALEO_MAXIMUM_FORK_DEPTH {
            warn!("Dropping {peer_ip} for an incorrect maximum fork depth of {}", message.fork_depth);
            return false;
        }
        // // Perform the deferred non-blocking deserialization of the block header.
        // match block_header.deserialize().await {
        //     Ok(block_header) => {
        //         // If this node is not a sync node and is syncing, the peer is a sync node, and this node is ahead, proceed to disconnect.
        //         if E::NODE_TYPE != NodeType::Beacon
        //             && E::status().is_syncing()
        //             && node_type == NodeType::Beacon
        //             && state.ledger().reader().latest_cumulative_weight() > block_header.cumulative_weight()
        //         {
        //             trace!("Disconnecting from {} (ahead of sync node)", peer_ip);
        //             break;
        //         }
        //
        //         // Update peer's block height.
        //         peer.block_height = block_header.height();
        //     }
        //     Err(error) => warn!("[Ping] {}", error),
        // }

        // Update the version of the peer.
        *peer.version.write().await = message.version;
        // Update the node type of the peer.
        *peer.node_type.write().await = message.node_type;
        // Update the status of the peer.
        *peer.status.write().await = message.status;

        // // Determine if the peer is on a fork (or unknown).
        // let is_fork = match state.ledger().reader().get_block_hash(peer.block_height) {
        //     Ok(expected_block_hash) => Some(expected_block_hash != block_hash),
        //     Err(_) => None,
        // };
        let is_fork = Some(false);

        // Send a `Pong` message to the peer.
        if let Err(error) = peer.send(Message::Pong(Pong { is_fork })).await {
            warn!("[Pong] {error}");
        }
        true
    }

    async fn pong(_message: Pong, peer_ip: SocketAddr, router: &Router<N>) -> bool {
        // // Perform the deferred non-blocking deserialization of block locators.
        // let request = match block_locators.deserialize().await {
        //     // Route the `Pong` to the ledger.
        //     Ok(block_locators) => LedgerRequest::Pong(peer_ip, peer.node_type, peer.status, is_fork, block_locators),
        //     // Route the `Failure` to the ledger.
        //     Err(error) => LedgerRequest::Failure(peer_ip, format!("{}", error)),
        // };
        //
        // // Route the request to the ledger.
        // if let Err(error) = state.ledger().router().send(request).await {
        //     warn!("[Pong] {}", error);
        // }

        // Spawn an asynchronous task for the `Ping` request.
        let router = router.clone();
        spawn_task!(Self, Self::resources().procure_id(), {
            // Sleep for the preset time before sending a `Ping` request.
            tokio::time::sleep(Duration::from_secs(Router::<N>::PING_SLEEP_IN_SECS)).await;

            // Send a `Ping` request to the peer.
            let message = Message::Ping(Ping {
                version: Message::<N>::VERSION,
                fork_depth: ALEO_MAXIMUM_FORK_DEPTH,
                node_type: Self::NODE_TYPE,
                status: Self::status().get(),
            });
            if let Err(error) = router.process(RouterRequest::MessageSend(peer_ip, message)).await {
                warn!("[Ping] {error}");
            }
        });
        true
    }

    async fn puzzle_request(&self, peer_ip: SocketAddr, _router: &Router<N>) -> bool {
        debug!("Disconnecting peer {peer_ip} for the following reason: {:?}", DisconnectReason::ProtocolViolation);
        false
    }

    async fn puzzle_response(&self, _message: PuzzleResponse<N>, peer_ip: SocketAddr) -> bool {
        debug!("Disconnecting peer {peer_ip} for the following reason: {:?}", DisconnectReason::ProtocolViolation);
        false
    }

    async fn unconfirmed_block(
        &self,
        message: UnconfirmedBlock<N>,
        peer_ip: SocketAddr,
        peer: &Peer<N>,
        router: &Router<N>,
    ) -> bool {
        // Drop the peer, if they have sent more than 5 unconfirmed blocks in the last 5 seconds.
        let frequency = peer
            .seen_inbound_blocks
            .read()
            .await
            .values()
            .filter(|t| t.elapsed().unwrap_or_default().as_secs() <= 5)
            .count();
        if frequency >= 10 {
            warn!("Dropping {peer_ip} for spamming unconfirmed blocks (frequency = {frequency})");
            // Send a `PeerRestricted` message.
            if let Err(error) = router.process(RouterRequest::PeerRestricted(peer_ip)).await {
                warn!("[PeerRestricted] {error}");
            }
            return false;
        }

        // Acquire the lock on the seen inbound blocks.
        let mut seen_inbound_blocks = peer.seen_inbound_blocks.write().await;

        // Retrieve the last seen timestamp of the received block.
        let last_seen = seen_inbound_blocks.entry(message.block_hash).or_insert(SystemTime::UNIX_EPOCH);
        let is_router_ready = last_seen.elapsed().unwrap_or_default().as_secs() > Router::<N>::RADIO_SILENCE_IN_SECS;

        // Update the timestamp for the received block.
        seen_inbound_blocks.insert(message.block_hash, SystemTime::now());

        // Drop the lock on the seen inbound blocks.
        drop(seen_inbound_blocks);

        // // Ensure the unconfirmed block is at least within 2 blocks of the latest block height,
        // // and no more that 2 blocks ahead of the latest block height.
        // // If it is stale, skip the routing of this unconfirmed block to the ledger.
        // let latest_block_height = state.ledger().reader().latest_block_height();
        // let lower_bound = latest_block_height.saturating_sub(2);
        // let upper_bound = latest_block_height.saturating_add(2);
        // let is_within_range = block_height >= lower_bound && block_height <= upper_bound;
        //
        // // Ensure the node is not peering.
        // let is_node_ready = !Self::status().is_peering();
        //
        // if !is_router_ready || !is_within_range || !is_node_ready {
        //     trace!("Skipping 'UnconfirmedBlock {}' from {}", block_height, peer_ip)
        // } else {
        //     // Perform the deferred non-blocking deserialization of the block.
        //     let request = match block.deserialize().await {
        //         // Ensure the claimed block height and block hash matches in the deserialized block.
        //         Ok(block) => match block_height == block.height() && block_hash == block.hash() {
        //             // Route the `UnconfirmedBlock` to the ledger.
        //             true => LedgerRequest::UnconfirmedBlock(peer_ip, block),
        //             // Route the `Failure` to the ledger.
        //             false => LedgerRequest::Failure(peer_ip, "Malformed UnconfirmedBlock message".to_string())
        //         },
        //         // Route the `Failure` to the ledger.
        //         Err(error) => LedgerRequest::Failure(peer_ip, format!("{}", error)),
        //     };
        //
        //     // Route the request to the ledger.
        //     if let Err(error) = state.ledger().router().send(request).await {
        //         warn!("[UnconfirmedBlock] {}", error);
        //     }
        // }
        true
    }

    async fn unconfirmed_solution(
        &self,
        message: UnconfirmedSolution<N>,
        peer_ip: SocketAddr,
        router: &Router<N>,
    ) -> bool {
        // Prepare the full message.
        let full_message = Message::UnconfirmedSolution(message.clone());

        // Perform the deferred non-blocking deserialization of the solution.
        match message.solution.deserialize().await {
            Ok(solution) => {
                // Update the timestamp for the unconfirmed solution.
                let seen_before = router
                    .seen_unconfirmed_solutions
                    .write()
                    .await
                    .insert(solution.commitment(), SystemTime::now())
                    .is_some();
                // Determine whether to propagate the solution.
                let should_propagate = !seen_before;

                if !should_propagate {
                    trace!("Skipping 'UnconfirmedSolution {}' from {peer_ip}", solution.commitment().0);
                } else {
                    // Propagate the `UnconfirmedSolution`.
                    let request = RouterRequest::MessagePropagate(full_message, vec![peer_ip]);
                    if let Err(error) = router.process(request).await {
                        warn!("[UnconfirmedSolution] {error}");
                    }
                }
            }
            Err(error) => warn!("[UnconfirmedSolution] {error}"),
        }
        true
    }

    async fn unconfirmed_transaction(
        &self,
        message: UnconfirmedTransaction<N>,
        peer_ip: SocketAddr,
        router: &Router<N>,
    ) -> bool {
        // Prepare the full message.
        let full_message = Message::UnconfirmedTransaction(message.clone());

        // Perform the deferred non-blocking deserialization of the transaction.
        match message.transaction.deserialize().await {
            Ok(transaction) => {
                // Update the timestamp for the unconfirmed transaction.
                let seen_before = router
                    .seen_unconfirmed_transactions
                    .write()
                    .await
                    .insert(transaction.id(), SystemTime::now())
                    .is_some();
                // Determine whether to propagate the transaction.
                let should_propagate = !seen_before;

                if !should_propagate {
                    trace!("Skipping 'UnconfirmedTransaction {}' from {peer_ip}", transaction.id());
                } else {
                    // Propagate the `UnconfirmedTransaction`.
                    let request = RouterRequest::MessagePropagate(full_message, vec![peer_ip]);
                    if let Err(error) = router.process(request).await {
                        warn!("[UnconfirmedTransaction] {error}");
                    }
                }
            }
            Err(error) => warn!("[UnconfirmedTransaction] {error}"),
        }
        true
    }
}
