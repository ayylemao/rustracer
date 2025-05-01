use std::sync::Arc;

use super::Shape;
use crate::matrix::{Matrix, SqMatrix};

pub struct Group {
    pub children: Vec<Arc<dyn Shape + Send + Sync>>,
    pub transfom: SqMatrix<4>,
}

impl Group {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            transfom: Matrix::eye(),
        }
    }
}

#[cfg(test)]
pub mod tests {

    #[test]
    fn create_group() {}
}
