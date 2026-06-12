use crate::core::range::uniform_u64_inclusive;
use crate::core::source::Source;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Domino {
    pub left: u8,
    pub right: u8,
}

impl std::fmt::Display for Domino {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}|{}]", self.left, self.right)
    }
}

fn double_n_set(n: u8) -> Vec<Domino> {
    let mut set = Vec::new();
    for left in 0..=n {
        for right in left..=n {
            set.push(Domino { left, right });
        }
    }
    set
}

/// Draw `count` dominoes from a double-n set without replacement.
pub fn draw_dominoes(
    source: &mut dyn Source,
    n: u8,
    count: usize,
) -> Result<Vec<Domino>, crate::core::SourceError> {
    let mut set = double_n_set(n);
    let count = count.min(set.len());

    // Fisher-Yates sample.
    for i in (1..set.len()).rev() {
        let j = uniform_u64_inclusive(source, 0, i as u64)? as usize;
        set.swap(i, j);
    }

    Ok(set.into_iter().take(count).collect())
}
