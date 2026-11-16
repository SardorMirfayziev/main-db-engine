// A leaf stores a count, then each pair one after another.
// Layout: | nkeys 2B | klen 2B | vlen 2B | key | val | klen 2B | ...  (little-endian)

// Build a leaf: write the pair count, then each pair's body.
pub fn build_leaf(pairs: Vec<(Vec<u8>, Vec<u8>)>) -> Vec<u8> {
    let mut leaf: Vec<u8> = Vec::new();
    // First 2 bytes: the pair count, little-endian.
    let number_of_pairs_as_u16 = pairs.len() as u16;
    leaf.extend_from_slice(&number_of_pairs_as_u16.to_le_bytes());
    // Then each pair: key length, value length, key bytes, value bytes.
    let mut pair_index = 0;
    while pair_index < pairs.len() {
        let key_bytes = &pairs[pair_index].0;
        let value_bytes = &pairs[pair_index].1;
        let key_length_as_u16 = key_bytes.len() as u16;
        let value_length_as_u16 = value_bytes.len() as u16;
        leaf.extend_from_slice(&key_length_as_u16.to_le_bytes());
        leaf.extend_from_slice(&value_length_as_u16.to_le_bytes());
        leaf.extend_from_slice(key_bytes);
        leaf.extend_from_slice(value_bytes);
        pair_index += 1;
    }
    leaf
}

// Read how many pairs the leaf holds from its first 2 bytes.
pub fn number_of_pairs(leaf: &Vec<u8>) -> u16 {
    u16::from_le_bytes([leaf[0], leaf[1]])
}

// Read one pair starting at byte_pos. Return the key, the value, and the byte
// position where the next pair starts (byte-cursor, decision D8).
pub fn read_pair_at(leaf: &Vec<u8>, byte_pos: usize) -> (Vec<u8>, Vec<u8>, usize) {
    // First 2 bytes at byte_pos: the key length, little-endian.
    let key_length = u16::from_le_bytes([leaf[byte_pos], leaf[byte_pos + 1]]) as usize;
    // Next 2 bytes: the value length, little-endian.
    let value_length = u16::from_le_bytes([leaf[byte_pos + 2], leaf[byte_pos + 3]]) as usize;
    // The key bytes come right after the two lengths.
    let key_start = byte_pos + 4;
    let value_start = key_start + key_length;
    let value_end = value_start + value_length;
    // Copy out the key and the value bytes.
    let key_bytes = leaf[key_start..value_start].to_vec();
    let value_bytes = leaf[value_start..value_end].to_vec();
    // The next pair starts right after this value.
    let next_pos = value_end;
    (key_bytes, value_bytes, next_pos)
}

#[test]
fn empty_leaf_has_zero_pairs() {
    let leaf = build_leaf(vec![]);
    assert_eq!(number_of_pairs(&leaf), 0);
}

#[test]
fn one_pair_leaf_has_one_pair() {
    let leaf = build_leaf(vec![(b"fox".to_vec(), b"9".to_vec())]);
    assert_eq!(number_of_pairs(&leaf), 1);
}

#[test]
fn one_pair_leaf_has_exact_bytes() {
    // (fox, 9) -> nkeys=1, klen=3, vlen=1, "fox" (66 6f 78), "9" (39).
    let leaf = build_leaf(vec![(b"fox".to_vec(), b"9".to_vec())]);
    assert_eq!(
        leaf,
        vec![0x01, 0x00, 0x03, 0x00, 0x01, 0x00, 0x66, 0x6f, 0x78, 0x39]
    );
}

#[test]
fn read_back_the_stored_pair() {
    // Build a one-pair leaf, then read pair 0 back (it starts at byte 2).
    let leaf = build_leaf(vec![(b"fox".to_vec(), b"9".to_vec())]);
    let (key_bytes, value_bytes, next_pos) = read_pair_at(&leaf, 2);
    assert_eq!(key_bytes, b"fox".to_vec());
    assert_eq!(value_bytes, b"9".to_vec());
    assert_eq!(next_pos, 10);
}
