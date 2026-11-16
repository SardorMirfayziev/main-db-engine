// A leaf starts with a 2-byte count: how many pairs it holds.
// Layout so far: | nkeys 2B | ...  (little-endian)

// Build a leaf that records only how many pairs it holds.
pub fn build_leaf(pairs: Vec<(Vec<u8>, Vec<u8>)>) -> Vec<u8> {
    let mut leaf: Vec<u8> = Vec::new();
    // First 2 bytes: the pair count, little-endian.
    let number_of_pairs_as_u16 = pairs.len() as u16;
    leaf.extend_from_slice(&number_of_pairs_as_u16.to_le_bytes());
    leaf
}

// Read how many pairs the leaf holds from its first 2 bytes.
pub fn number_of_pairs(leaf: &Vec<u8>) -> u16 {
    u16::from_le_bytes([leaf[0], leaf[1]])
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
