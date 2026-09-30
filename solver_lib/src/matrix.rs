use std::collections::HashSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Dimension {
    pub x: usize,
    pub y: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Matrix {
    matrix: Vec<Vec<f64>>,
    dimension: Dimension,
}

impl Matrix {
    pub fn new(dimension: Dimension) -> Self {
        let matrix = vec![vec![0.0; dimension.x]; dimension.y];

        Self { matrix, dimension }
    }

    pub fn get_dimensions(&self) -> &Dimension {
        &self.dimension
    }

    pub fn get_cell(&self, x: usize, y: usize) -> Option<f64> {
        self.matrix.get(y)?.get(x).copied()
    }

    pub fn set_cell(&mut self, x: usize, y: usize, value: f64) -> Option<()> {
        let cell = self.matrix.get_mut(y)?.get_mut(x)?;
        *cell = value;
        Some(())
    }

    pub fn get_row(&self, y: usize) -> Option<&[f64]> {
        self.matrix.get(y).map(|row| row.as_slice())
    }

    pub fn get_row_mut(&mut self, y: usize) -> Option<&mut [f64]> {
        self.matrix.get_mut(y).map(|row| row.as_mut_slice())
    }

    pub fn get_col(&self, x: usize) -> impl Iterator<Item = f64> + '_ {
        self.matrix
            .iter()
            .filter_map(move |row| row.get(x).copied())
    }

    pub fn rows(&self) -> impl Iterator<Item = &[f64]> {
        self.matrix.iter().map(|row| row.as_slice())
    }

    pub fn rows_mut(&mut self) -> impl Iterator<Item = &mut [f64]> {
        self.matrix.iter_mut().map(|row| row.as_mut_slice())
    }

    pub fn iter(&self) -> impl Iterator<Item = (usize, usize, f64)> + '_ {
        self.matrix
            .iter()
            .enumerate()
            .flat_map(|(y, row)| row.iter().enumerate().map(move |(x, &val)| (x, y, val)))
    }

    pub fn is_row_echolon_form(&self) -> bool {
        let mut set = HashSet::<usize>::new();

        for i in 0..self.dimension.x {
            let mut one_detected: Option<usize> = Option::None;

            // row echolon form must be 0s or 1s and it only needs 1 1 and the rest 0s
            for j in 0..self.dimension.y {
                let value = self.get_cell(i, j);
                if value.eq(&Some(0.0)) {
                    continue;
                } else if value.eq(&Some(1.0)) {
                    // only set 1 detected to the row if not already detected
                    // otherwise this column doesn't work
                    if one_detected == Option::None {
                        one_detected = Option::Some(j);
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }

            // insert the row found into the set
            match one_detected {
                Some(value) => {
                    set.insert(value);
                }
                None => {
                    continue;
                }
            }
        }

        set.iter().count() == self.dimension.y
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matrix_creation_and_dimensions() {
        let dim = Dimension { x: 3, y: 2 };
        let matrix = Matrix::new(dim);

        assert_eq!(*matrix.get_dimensions(), dim);
        assert_eq!(matrix.get_cell(0, 0), Some(0.0));
        assert_eq!(matrix.get_cell(2, 1), Some(0.0));
        assert_eq!(matrix.get_cell(3, 1), None);
    }

    #[test]
    fn test_matrix_set_and_get_cell() {
        let mut matrix = Matrix::new(Dimension { x: 3, y: 3 });

        assert_eq!(matrix.set_cell(1, 2, 42.0), Some(()));
        assert_eq!(matrix.get_cell(1, 2), Some(42.0));
        assert_eq!(matrix.set_cell(5, 5, 10.0), None);
    }

    #[test]
    fn test_row_and_column_iteration() {
        let mut matrix = Matrix::new(Dimension { x: 2, y: 2 });
        matrix.set_cell(0, 0, 1.0);
        matrix.set_cell(1, 0, 2.0);
        matrix.set_cell(0, 1, 3.0);
        matrix.set_cell(1, 1, 4.0);

        // Row iteration
        let row_0 = matrix.get_row(0).unwrap();
        assert_eq!(row_0, &[1.0, 2.0]);

        // Column iteration
        let col_1: Vec<f64> = matrix.get_col(1).collect();
        assert_eq!(col_1, vec![2.0, 4.0]);
    }

    #[test]
    fn test_row_mut_operations() {
        let mut matrix = Matrix::new(Dimension { x: 3, y: 2 });
        if let Some(row) = matrix.get_row_mut(0) {
            for val in row.iter_mut() {
                *val = 5.0;
            }
        }

        assert_eq!(matrix.get_row(0).unwrap(), &[5.0, 5.0, 5.0]);
        assert_eq!(matrix.get_row(1).unwrap(), &[0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_matrix_iter_all() {
        let mut matrix = Matrix::new(Dimension { x: 2, y: 2 });
        matrix.set_cell(0, 0, 10.0);
        matrix.set_cell(1, 1, 20.0);

        let elements: Vec<(usize, usize, f64)> = matrix.iter().collect();
        assert_eq!(
            elements,
            vec![(0, 0, 10.0), (1, 0, 0.0), (0, 1, 0.0), (1, 1, 20.0),]
        );
    }
}
