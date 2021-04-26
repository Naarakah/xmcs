//! Check whether a sequence is a subsequence of another
//!
//! This module contains a structure used to precompute and the answer in
//! constant time whether the tail of a sequence is a subsequence of another

#[doc(hidden)]
const fn distance(a: usize, b: usize) -> usize {
    if a > b {
        a - b
    } else {
        b - a
    }
}

/// Struct used to precompute whether a sequence is a subsequence of
/// another
///
/// Given two sequences `s1` and `s2` and an integer `delta`,
/// this struct holds data  used to answer the following question
/// in constant time:
///
/// for any `i`, `j` indices of `s1`, `s2` such that the distance between
/// `i` and `j` is less than `delta`, is `s1[i..]` is a subsequence of
/// `s2[j..]` or `s2[j..]` a subsequence of `s1[i..]`.
///
/// It only needs to do a precomputation in time `O(|s1| * delta)`
/// beforehand.
///
/// # Examples
/// ```
/// # use xmcs::substr::SubString;
/// // Precompute results
/// let res = SubString::new(b"ABCABC", b"ACBAC", 5);
///
/// // "BC" is a substring of "BAC"
/// assert_eq!(true, res.is_substring_at(4, 2));
///
/// // "BAC" is a substring of "BCABC"
/// assert_eq!(true, res.is_substring_at(1, 2));
///
/// // "ABC" is not a substring of "CBAC"
/// assert_eq!(false, res.is_substring_at(3, 1));
/// ```
pub struct SubString {
    d1: usize,
    d2: usize,
    delta: usize,
    table: Vec<bool>, // TODO: use something more optimized for this?
}

impl SubString {
    /// Do the precomputations and returns a struct containing the
    /// resulting data.
    /// Use [`is_substring_at`] on the result to get the answer.
    ///
    /// The following property must hold: `||s1| - |s2|| <= delta`,
    /// otherwise panic.
    ///
    /// [`is_substring_at`]: `SubString::is_substring_at`
    pub fn new<T: Eq>(s1: &[T], s2: &[T], delta: usize) -> Self {
        Self::compute(s1, s2, delta)
    }

    /// Returns whether the tail of one of the sequence is a subsequence of
    /// the tail of the other sequence
    ///
    /// If `|s1| - i < |s2| - j`, returns whether `s1[i..]` is a
    /// subsequence of `s2[j..]`
    ///
    /// If `|s2| - j < |s1| - i`, returns whether `s2[j..]` is a
    /// subsequence of `s1[i..]`
    ///
    /// If `|s1| - i = |s2| - j`, returns whether `s1[i..]` and `s2[j..]`
    /// are equal.
    ///
    /// Runs in constant time because results are precomputed at
    /// construction
    ///
    /// # Panics
    /// Panics if the distance between `i` and `j` is not less than the
    /// `delta` parameter used at construction.
    ///
    /// # Example
    ///
    /// ```
    /// # use xmcs::substr::SubString;
    /// let res = SubString::new(b"CABC", b"DBABDCD", 3);
    ///
    /// // "ABC" is a substring of "ABDCD"
    /// assert_eq!(true, res.is_substring_at(1, 2));
    ///
    /// // "BC" is not a substring of "DCD"
    /// assert_eq!(false, res.is_substring_at(2, 4));
    /// ```
    pub fn is_substring_at(&self, i: usize, j: usize) -> bool {
        assert!(i <= self.d1);
        assert!(j <= self.d2);
        assert!(distance(i, j) <= self.delta);

        // At least one of the sequence is empty
        if i == self.d1 || j == self.d2 {
            return true;
        }

        self.table[self.index(i, j)]
    }

    /// Returns whether the tail of one of the sequence is a subsequence
    /// of the tail of the other sequence, indexing from the end of the
    /// sequences.
    /// See [`is_substring_at`].
    ///
    /// [`is_substring_at`]: `SubString::is_substring_at`
    pub fn is_substring_from_end(&self, end_i: usize, end_j: usize) -> bool {
        let i = self.d1 - end_i;
        let j = self.d2 - end_j;

        self.is_substring_at(i, j)
    }

    #[doc(hidden)]
    fn compute<T: Eq>(s1: &[T], s2: &[T], delta: usize) -> Self {
        use std::cmp::Ordering;

        let d1 = s1.len();
        let d2 = s2.len();

        assert!(distance(d1, d2) <= delta);

        let mut res = Vec::new();
        res.resize(d1 * (2 * delta + 1), false);

        let index = |i: usize, j: usize| Self::index_with(i, j, delta);

        for i in (0..d1).rev() {
            // begin loop where we already have |i - j| <= delta
            let start = (i + delta + 1).min(d2);
            for j in (0..start).rev() {
                // if |i - j| > delta, jump to next i
                if distance(i, j) > delta {
                    break;
                }

                let end_i = d1 - i - 1;
                let end_j = d2 - j - 1;
                // we have 0 <= i < d1 and 0 <= j < d2
                // thus 0 <= d1 - i - 1 < d1 and 0 <= j < d2

                // is s1[i..] a substr of s2[j..]? (or the other way around)
                let is_substr = match Ord::cmp(&end_i, &end_j) {
                    // s1[d1-1..] == s2[d2-1..] iff s1[d1-1] == s2[d2-1]
                    Ordering::Equal if end_i == 0 =>
                        s1[i] == s2[j],

                    // s1[d1-k..] == s2[d2-k..] iff
                    // s1[d1-k+1..] == s2[d2-k+1..] and s1[d1-k] == s2[d2-k]
                    Ordering::Equal =>
                        res[index(i + 1, j + 1)] && s1[i] == s2[j],

                    // s1[d1-1..] is a substring of s2[d2-k..] iff
                    // s1[d1-1..] is a substring of s2[d2-k+1..]
                    // or s1[d1-1] == s2[d2-k]
                    Ordering::Less if end_i == 0 =>
                        res[index(i, j + 1)] || s1[i] == s2[j],

                    // s1[d1-i..] is a substring of s2[d2-j..] iff
                    // s1[d1-i..] is a substring of s2[d2-j+1..] or
                    // s1[d1-i+1..] is a substring of s2[d2-j+1..]
                    //   and s1[d1-i] == s2[d2-j]
                    Ordering::Less => 
                        res[index(i, j + 1)] || (res[index(i + 1, j + 1)] && s1[i] == s2[j]),

                    // etc...
                    Ordering::Greater if end_j == 0 =>
                        res[index(i + 1, j)] || s1[i] == s2[j],
                    Ordering::Greater =>
                        res[index(i + 1, j)] || (res[index(i + 1, j + 1)] && s1[i] == s2[j]),
                };

                res[index(i, j)] = is_substr;
            }
        }

        Self {
            d1,
            d2,
            delta,
            table: res,
        }
    }

    #[doc(hidden)]
    const fn index_with(i: usize, j: usize, delta: usize) -> usize {
        j + delta + i * 2 * delta
    }

    #[doc(hidden)]
    const fn index(&self, i: usize, j: usize) -> usize {
        Self::index_with(i, j, self.delta)
    }
}

// === Tests ===

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_index_mapping() {
        let i = SubString::index_with(0, 0, 4);
        assert_eq!(4, i);

        let i = SubString::index_with(0, 1, 4);
        assert_eq!(5, i);

        let i = SubString::index_with(1, 0, 4);
        assert_eq!(12, i);

        let i = SubString::index_with(34, 42, 23);
        assert_eq!(1629, i);
    }

    #[test]
    fn test_substring_1() {
        let res = SubString::new(b"ACBBABCBCACBACBCBBABBCAC", b"ABCBABAAABACBABCABCA", 8);

        assert_eq!(false, res.is_substring_at(22, 18));
        assert_eq!(true, res.is_substring_at(21, 18));
        assert_eq!(true, res.is_substring_at(20, 18));
        assert_eq!(true, res.is_substring_at(19, 18));
        assert_eq!(true, res.is_substring_at(15, 18));
        assert_eq!(true, res.is_substring_at(14, 18));

        assert_eq!(false, res.is_substring_at(21, 17));
        assert_eq!(true, res.is_substring_at(20, 17));
        assert_eq!(true, res.is_substring_at(14, 17));
        assert_eq!(true, res.is_substring_at(13, 17));

        assert_eq!(false, res.is_substring_at(21, 16));
        assert_eq!(true, res.is_substring_at(21, 15));
        assert_eq!(true, res.is_substring_at(21, 14));
        assert_eq!(true, res.is_substring_at(21, 13));

        assert_eq!(false, res.is_substring_at(8, 9));
        assert_eq!(true, res.is_substring_at(7, 9));
        assert_eq!(true, res.is_substring_at(5, 9));

        assert_eq!(false, res.is_substring_at(0, 0));
        assert_eq!(false, res.is_substring_at(8, 4));
        assert_eq!(false, res.is_substring_at(16, 10));
    }

    #[test]
    fn test_substring_2() {
        let res = SubString::new(b"BCABCDAABCD", b"ABCDABCDABCDABCD", 6);

        assert_eq!(true, res.is_substring_at(10, 15));
        assert_eq!(true, res.is_substring_at(9, 14));
        assert_eq!(true, res.is_substring_at(8, 13));
        assert_eq!(true, res.is_substring_at(7, 12));
        assert_eq!(false, res.is_substring_at(6, 11));

        assert_eq!(false, res.is_substring_at(6, 10));
        assert_eq!(false, res.is_substring_at(6, 9));
        assert_eq!(true, res.is_substring_at(6, 8));
    }
}
