use std::cmp::{Ordering};
use crate::tree::art::util::PrefixMatch::{Equal, NoMatch, PartialMatch};

#[derive(PartialEq, Debug)]
pub(crate) enum PrefixMatch {
    Equal,
    PartialMatch(Vec<u8>),
    NoMatch
}

pub(crate) fn cmp_prefix(a: &[u8], b: &[u8]) -> PrefixMatch {
    let m = a.len();
    let n = b.len();
    let (maybe_eq, len) =match m.cmp(&n) {
        Ordering::Less => (false, m),
        Ordering::Equal => (true, m),
        Ordering::Greater => (false, n)
    };

    for i in 0..len {
        if a[i] != b[i] {
            return if i == 0 {
                NoMatch
            } else {
                PartialMatch(Vec::from(&a[..i]))
            }
        }
    }

    return if maybe_eq {
        Equal
    } else {
        PartialMatch(Vec::from(&a[..len]))
    }
}

#[cfg(test)]
mod tests {
    use crate::tree::art::util::cmp_prefix;
    use crate::tree::art::util::PrefixMatch::{Equal, NoMatch, PartialMatch};

    #[test]
    fn test_cmp_prefix() {
        assert_eq!(cmp_prefix(&[0, 1, 2], &[0, 1, 2]), Equal);
        assert_eq!(cmp_prefix(&[0, 1], &[0, 1, 2]), PartialMatch(vec![0, 1]));
        assert_eq!(cmp_prefix(&[0, 1, 2], &[0, 1]), PartialMatch(vec![0, 1]));
        assert_eq!(cmp_prefix(&[0, 1, 2], &[0, 1, 3]), PartialMatch(vec![0, 1]));
        assert_eq!(cmp_prefix(&[0, 1, 2], &[1, 2, 3]), NoMatch);
        assert_eq!(cmp_prefix(&[0, 1, 2], &[1]), NoMatch);
    }
}