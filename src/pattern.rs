use std::ops::Deref;

// TODO: Bench
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Pattern<const N: usize>([Option<u8>; N]);

impl<const N: usize> Deref for Pattern<N> {
    type Target = [Option<u8>; N];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const N: usize> From<[Option<u8>; N]> for Pattern<N> {
    fn from(value: [Option<u8>; N]) -> Self {
        Self(value)
    }
}

impl<const N: usize> PartialEq<&[u8]> for Pattern<N> {
    fn eq(&self, other: &&[u8]) -> bool {
        for (index, element) in self.0.iter().enumerate() {
            match element {
                Some(value) => {
                    if other.get(index).map_or(true, |e| e != value) {
                        return false;
                    }
                }
                None => continue,
            }
        }

        if other.get(N).is_some() {
            return false;
        }

        return true;
    }
}

impl<const N: usize> PartialEq<Pattern<N>> for &[u8] {
    fn eq(&self, other: &Pattern<N>) -> bool {
        other == self
    }
}

impl<const N: usize> PartialEq<&[u8; N]> for Pattern<N> {
    fn eq(&self, other: &&[u8; N]) -> bool {
        for (index, element) in self.0.iter().enumerate() {
            match element {
                Some(value) => {
                    if other.get(index).map_or(true, |e| e != value) {
                        return false;
                    }
                }
                None => continue,
            }
        }

        return true;
    }
}

impl<const N: usize> PartialEq<Pattern<N>> for &[u8; N] {
    fn eq(&self, other: &Pattern<N>) -> bool {
        other == self
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn comparing_to_slice_u8_defined_size() {
        let a = super::Pattern::from([Some(8), None, Some(20)]);
        let b = [8u8, 11u8, 20u8];

        assert_eq!(a, &b);
        assert_eq!(&b, a);
    }

    #[test]
    fn comparing_to_slice_u8_unknown_size() {
        let a = super::Pattern::from([Some(8), None, Some(20)]);
        let b = [8u8, 11u8, 20u8];
        let b1 = &b as &[u8];
        let c = [8u8, 11u8, 20u8, 0u8];
        let c1 = &c as &[u8];

        assert_eq!(a, b1);
        assert_eq!(b1, a);
        assert_ne!(a, c1);
        assert_ne!(c1, a);
    }
}
