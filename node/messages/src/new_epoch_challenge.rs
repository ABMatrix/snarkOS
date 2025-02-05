// Copyright (C) 2019-2023 Aleo Systems Inc.
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

use super::*;
use snarkvm::prelude::Address;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewEpochChallenge<N: Network> {
    pub block_height: u32,
    pub proof_target: u64,
    pub address: Address<N>,
    pub epoch_challenge: EpochChallenge<N>,
}

impl<N: Network> MessageTrait for NewEpochChallenge<N> {
    /// Returns the message name.
    #[inline]
    fn name(&self) -> String {
        "NewEpochChallenge".to_string()
    }

    /// Serializes the message into the buffer.
    #[inline]
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_all(&self.block_height.to_le_bytes())?;
        writer.write_all(&self.proof_target.to_le_bytes())?;
        writer.write_all(&self.address.to_bytes_le()?)?;
        writer.write_all(&self.epoch_challenge.to_bytes_le()?)?;
        Ok(())
    }

    /// Deserializes the given buffer into a message.
    #[inline]
    fn deserialize(bytes: BytesMut) -> Result<Self> {
        let mut reader = bytes.reader();
        Ok(Self {
            block_height: bincode::deserialize_from(&mut reader)?,
            proof_target: bincode::deserialize_from(&mut reader)?,
            address: bincode::deserialize_from(&mut reader)?,
            epoch_challenge: EpochChallenge::read_le(&mut reader)?,
        })
    }
}
