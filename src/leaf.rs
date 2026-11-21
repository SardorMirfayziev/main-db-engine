#[allow(dead_code)]
const COUNT_BYTES: usize = 2;
// Bytes for one length field (klen or vlen).
#[allow(dead_code)]
const LEN_BYTES: usize = 2;

// The leaf itself: it owns its on-disk bytes. Every leaf operation is a method
// on this type, so the bytes and the code that reads them stay together.
pub struct Leaf {
    // The raw leaf bytes in the layout above.
    pub bytes: Vec<u8>,
}

impl Leaf {
    // ==== layer 1: build ====

    // Build a leaf: write the pair count, then each pair's body.
    pub fn build(pairs: Vec<(Vec<u8>, Vec<u8>)>) -> Leaf {
        let mut bytes: Vec<u8> = Vec::new();
        // First 2 bytes: the pair count, little-endian.
        let number_of_pairs_as_u16 = pairs.len() as u16;
        bytes.extend_from_slice(&number_of_pairs_as_u16.to_le_bytes());
        // Then each pair: key length, value length, key bytes, value bytes.
        let mut pair_index = 0;
        while pair_index < pairs.len() {
            let key_bytes = &pairs[pair_index].0;
            let value_bytes = &pairs[pair_index].1;
            let key_length_as_u16 = key_bytes.len() as u16;
            let value_length_as_u16 = value_bytes.len() as u16;
            bytes.extend_from_slice(&key_length_as_u16.to_le_bytes());
            bytes.extend_from_slice(&value_length_as_u16.to_le_bytes());
            bytes.extend_from_slice(key_bytes);
            bytes.extend_from_slice(value_bytes);
            pair_index += 1;
        }
        Leaf { bytes }
    }

    // ==== layer 2: read one pair ====

    // Read one pair starting at byte_pos. Return the key, the value, and the byte
    // position where the next pair starts (byte-cursor, decision D8).
    pub fn read_pair_at(&self, byte_pos: usize) -> (Vec<u8>, Vec<u8>, usize) {
        // First 2 bytes at byte_pos: the key length, little-endian.
        let key_length =
            u16::from_le_bytes([self.bytes[byte_pos], self.bytes[byte_pos + 1]]) as usize;
        // Next 2 bytes: the value length, little-endian.
        let value_length =
            u16::from_le_bytes([self.bytes[byte_pos + 2], self.bytes[byte_pos + 3]]) as usize;
        // The key bytes come right after the two lengths.
        let key_start = byte_pos + 4;
        let value_start = key_start + key_length;
        let value_end = value_start + value_length;
        // Copy out the key and the value bytes.
        let key_bytes = self.bytes[key_start..value_start].to_vec();
        let value_bytes = self.bytes[value_start..value_end].to_vec();
        // The next pair starts right after this value.
        let next_pos = value_end;
        (key_bytes, value_bytes, next_pos)
    }

    // ==== layer 3: read many ====

    // Read how many pairs the leaf holds from its first 2 bytes.
    pub fn number_of_pairs(&self) -> u16 {
        u16::from_le_bytes([self.bytes[0], self.bytes[1]])
    }

    // Find a value by its key. Walk each pair, compare keys, return the value on a
    // match. Walking uses the next_pos cursor from read_pair_at (decision D8).
    pub fn get(&self, wanted_key: &Vec<u8>) -> Option<Vec<u8>> {
        // How many pairs this leaf holds.
        let pair_count = self.number_of_pairs() as usize;
        // The first pair starts right after the 2-byte count.
        let mut byte_pos = 2;
        let mut pair_index = 0;
        while pair_index < pair_count {
            // Read this pair and where the next one starts.
            let (key_bytes, value_bytes, next_pos) = self.read_pair_at(byte_pos);
            // On a key match, hand back this pair's value.
            if key_bytes == *wanted_key {
                return Some(value_bytes);
            }
            // No match: step the cursor to the next pair.
            byte_pos = next_pos;
            pair_index += 1;
        }
        // Walked every pair without a match.
        None
    }

    pub fn insert(&self, key: &Vec<u8>, value: &Vec<u8>) -> Leaf {
        let nkeys = self.number_of_pairs();

        // Fresh buffer. Start with a placeholder count; we patch it at the end.
        let mut buffer: Vec<u8> = Vec::new();
        buffer.extend_from_slice(&0u16.to_le_bytes());

        let mut cur_byte_position = 2;
        let mut index = 0;
        let mut key_was_found = false;

        while &index < &nkeys {
            let (found_key, found_val, next_byte) = self.read_pair_at(cur_byte_position);

            if found_key == *key {
                // Matched: write this key with the NEW value.
                let given_key_len = key.len() as u16;
                let given_val_len = value.len() as u16;

                buffer.extend_from_slice(&given_key_len.to_le_bytes());
                buffer.extend_from_slice(&given_val_len.to_le_bytes());
                buffer.extend_from_slice(key);
                buffer.extend_from_slice(value);
                key_was_found = true;
            } else {
                // No match: copy the old pair unchanged.
                let found_key_len = found_key.len() as u16;
                let found_val_len = found_val.len() as u16;

                buffer.extend_from_slice(&found_key_len.to_le_bytes());
                buffer.extend_from_slice(&found_val_len.to_le_bytes());
                buffer.extend_from_slice(&found_key);
                buffer.extend_from_slice(&found_val);
            }

            cur_byte_position = next_byte;
            index = index + 1;
        }

        // Key not present: append it as a brand-new pair.
        if !key_was_found {
            let given_key_len = key.len() as u16;
            let given_val_len = value.len() as u16;

            buffer.extend_from_slice(&given_key_len.to_le_bytes());
            buffer.extend_from_slice(&given_val_len.to_le_bytes());
            buffer.extend_from_slice(key);
            buffer.extend_from_slice(value);
        }

        // Final count: same as before on a replace, one more on an append.
        let final_count = if key_was_found { nkeys } else { nkeys + 1 };
        let final_count_le = final_count.to_le_bytes();
        buffer[0] = final_count_le[0];
        buffer[1] = final_count_le[1];

        Leaf { bytes: buffer }
    }

    // Remove a key from the leaf and return a brand-new Leaf (D22 — copy-on-write).
    // Walk every old pair into a fresh buffer, copying all of them except the one
    // whose key matches. Patch the real count at the end. If no key matches, every
    // pair is copied, so the result is an unchanged copy (D23 — safe no-op).
    pub fn remove(&self, key_to_remove: &Vec<u8>) -> Leaf {
        let nkeys = self.number_of_pairs();

        // Fresh buffer. Start with a placeholder count; we patch it at the end.
        let mut buffer: Vec<u8> = Vec::new();
        buffer.extend_from_slice(&0u16.to_le_bytes());

        let mut cur_byte_position = 2;
        let mut index = 0;
        let mut kept_pair_count: u16 = 0;

        while &index < &nkeys {
            let (found_key, found_val, next_byte) = self.read_pair_at(cur_byte_position);

            if found_key == *key_to_remove {
                // Matched: skip it, so this pair is not copied into the new leaf.
            } else {
                // No match: copy the old pair unchanged.
                let found_key_len = found_key.len() as u16;
                let found_val_len = found_val.len() as u16;

                buffer.extend_from_slice(&found_key_len.to_le_bytes());
                buffer.extend_from_slice(&found_val_len.to_le_bytes());
                buffer.extend_from_slice(&found_key);
                buffer.extend_from_slice(&found_val);
                kept_pair_count = kept_pair_count + 1;
            }

            cur_byte_position = next_byte;
            index = index + 1;
        }

        // Patch the first two bytes with how many pairs we actually kept.
        let kept_count_le = kept_pair_count.to_le_bytes();
        buffer[0] = kept_count_le[0];
        buffer[1] = kept_count_le[1];

        Leaf { bytes: buffer }
    }
}
