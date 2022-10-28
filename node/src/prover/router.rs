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

use tokio::task;
use snarkos_node_messages::{ChallengeRequest, NewEpochChallenge};
use super::*;

#[async_trait]
impl<N: Network> Handshake for Prover<N> {}

#[async_trait]
impl<N: Network> Inbound<N> for Prover<N> {
    /// Saves the latest epoch challenge and latest block in the prover.
    async fn puzzle_response(&self, message: PuzzleResponse<N>, peer_ip: SocketAddr) -> bool {
        let epoch_challenge = message.epoch_challenge;
        match message.block.deserialize().await {
            Ok(block) => {
                trace!("received puzzle_response");
                let latest_epoch_challenge = self.latest_epoch_challenge.read().await.clone();
                if latest_epoch_challenge.is_none() || (latest_epoch_challenge.is_some() && epoch_challenge != latest_epoch_challenge.unwrap()) {
                    // Save the latest epoch challenge in the prover.
                    self.latest_epoch_challenge.write().await.replace(epoch_challenge.clone());
                    // Save the latest block in the prover.
                    self.latest_block.write().await.replace(block.clone());
                    trace!("latest_epoch_challenge updated");
                    let router = self.router.clone();
                    let address = self.account.address().clone();

                    if let Err(e) = router.process(RouterRequest::SendNewEpochChallenge(
                        Message::NewEpochChallenge(NewEpochChallenge {
                            proof_target: block.proof_target(),
                            address,
                            epoch_challenge: epoch_challenge.clone(),
                        }))
                    ).await {
                        warn!("[puzzle_response] {}", e);
                    }
                }
                true
            }
            Err(error) => {
                error!("Failed to deserialize the puzzle response from '{peer_ip}': {error}");
                false
            }
        }
    }
}

#[async_trait]
impl<N: Network> Outbound for Prover<N> {}
