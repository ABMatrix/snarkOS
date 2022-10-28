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

use super::*;

#[derive(Clone, Debug)]
pub struct NewEpochChallenge<N: Network> {
    pub proof_target: u64,
    pub latest_epoch_num: u32,
    pub previous_block_hash: N::BlockHash,
}

impl<N: Network> MessageTrait for NewEpochChallenge<N> {
    /// Returns the message name.
    #[inline]
    fn name(&self) -> &str {
        "NewEpochChallenge"
    }

    /// Serializes the message into the buffer.
    #[inline]
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_all(&self.proof_target.to_le_bytes())?;
        writer.write_all(&self.latest_epoch_num.to_le_bytes())?;
        writer.write_all(&self.previous_block_hash.to_bytes_le()?)?;
        Ok(())
    }

    /// Deserializes the given buffer into a message.
    #[inline]
    fn deserialize(bytes: BytesMut) -> Result<Self> {
        let mut reader = bytes.reader();
        Ok(Self {
            proof_target: bincode::deserialize_from(&mut reader)?,
            latest_epoch_num: bincode::deserialize_from(&mut reader)?,
            previous_block_hash: bincode::deserialize_from(&mut reader)?,
        })
    }
}
