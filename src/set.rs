use std::collections::HashSet;
use std::hash::Hash;

use crate::substr::SubString;

/// Compute an extended set of maximal common subsequences of all
/// the sequences in `seqs`, of size at least `len`.
pub fn xmcsk<T: Eq + Hash + Copy>(len: usize, seqs: &[&[T]]) -> HashSet<Vec<T>> {
    let k = seqs.len();
    let mut res = HashSet::new();

    if k == 1 {
        if seqs[0].len() >= len {
            res.insert(seqs[0].to_vec());
        }
    } else if k > 1 {
        let xmcs = xmcsk(len, &seqs[..(k-1)]);
        for s in xmcs.into_iter() {
            let substrings = xmcs2(len, &s, seqs[k-1]);
            res.extend(substrings.into_iter());
        }
    }

    res
}

/// Compute an extended set of maximal common subsequences of s1 and s2,
/// of size at least `len`.
////
//// Runs in `O(2^(Δ + δ) * n)` where `n = max(|s1|, |s2|)`, `m = min(|s1|, |s2|)`,
//// `Δ = n - len` and `δ = m - len`
pub fn xmcs2<T: Eq + Hash + Copy>(len: usize, s1: &[T], s2: &[T]) -> HashSet<Vec<T>> {
    let n = std::cmp::max(s1.len(), s2.len());
    let delta = n - len;

    let substring = SubString::new(s1, s2, delta);

    xmcs2_impl(len, s1, s2, &substring)
}

fn xmcs2_impl<T: Eq + Hash + Copy> (
    len: usize,
    s1: &[T], s2: &[T],
    substr: &SubString
) -> HashSet<Vec<T>>
{
    // Too much elements removed, no subsequence long enough here
    if len > s1.len() || len > s2.len()
        || s1.is_empty() || s2.is_empty() {
        return HashSet::new();
    }

    // One is a subsequence of another, return it
    if substr.is_substring_from_end(s1.len(), s2.len()) {
        let mut res = HashSet::new();
        if s1.len() < s2.len() {
            res.insert(s1.to_vec());
        } else {
            res.insert(s2.to_vec());
        }
        return res;
    }

    let u1 = s1[0];
    let u2 = s2[0];

    if u1 == u2 {
        // saturating_sub: do not undeflow at 0. The len is not important anymore
        // when it reaches 0 so this is not an issue.
        let res = xmcs2_impl(len.saturating_sub(1), &s1[1..], &s2[1..], substr);
        res.into_iter()
            .map(|mut s| { s.insert(0, u1); s}) // Very inefficient
            .collect::<HashSet<Vec<T>>>()
    } else {
        let res1 = xmcs2_impl(len, &s1[1..], s2, substr);
        let res2 = xmcs2_impl(len, s1, &s2[1..], substr);
        res1.into_iter()
            .chain(res2.into_iter())
            .collect::<HashSet<Vec<T>>>()
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_xmcs2() {
        let res = xmcs2(3, b"ABCD", b"ACBD");
        let mut expected = HashSet::new();
        expected.insert(b"ACD".to_vec());
        expected.insert(b"ABD".to_vec());

        assert!(expected.is_subset(&res));

        let res = xmcs2(5, b"AEBCDABCD", b"BADECABCD");
        let mut expected = HashSet::new();
        expected.insert(b"AECABCD".to_vec());
        expected.insert(b"ADABCD".to_vec());
        expected.insert(b"BCABCD".to_vec());
        expected.insert(b"BDABCD".to_vec());

        assert!(expected.is_subset(&res));
    }

    #[test]
    fn test_xmcsk() {
        let res = xmcsk(4, &[
            b"ADBCBAD",
            b"ADCBACD",
            b"ABDCABDA",
            b"BADBCBADC"
        ]);

        let mut expected = HashSet::new();
        expected.insert(b"ADCAD".to_vec());
        expected.insert(b"ABCD".to_vec());
        expected.insert(b"ACBD".to_vec());

        assert!(expected.is_subset(&res));
    }
}
