#[derive(Copy, Clone, Debug)]
pub struct Point {
    pub x: usize,
    pub y: usize,
}

impl PartialEq for Point {
    fn eq(&self, other: &Point) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl Point {
    pub fn get_offset(self, width: usize, height: usize, x: isize, y: isize) -> Option<Point> {
        if let Some(new_x) = Point::get_offset_dimension(width, self.x, x) {
            if let Some(new_y) = Point::get_offset_dimension(height, self.y, y) {
                Some(Point { x: new_x, y: new_y })
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_offset_dimension(max_dimension: usize, cur_val: usize, offset: isize) -> Option<usize> {
        let offset_val_maybe = if offset < 0 {
            cur_val.checked_sub(offset.abs() as usize)
        } else {
            cur_val.checked_add(offset.abs() as usize)
        };
        match offset_val_maybe {
            Some(offset_val) => {
                if offset_val >= max_dimension {
                    None
                } else {
                    Some(offset_val)
                }
            }
            None => None,
        }
    }

    pub fn get_abs_distance(&self, other: &Point) -> i32 {
        (self.x as i32 - other.x as i32).abs() + (self.y as i32 - other.y as i32).abs()
    }

    pub fn get_point_from_index(width: usize, index: usize) -> Point {
        Point {
            x: index % width,
            y: index / width,
        }
    }

    pub fn get_index_from_point(width: usize, point: &Point) -> usize {
        point.y * width + point.x
    }

    pub fn index_within_board(board: &[Vec<i8>], index: usize) -> bool {
        if board.is_empty() || board[0].is_empty() {
            return false;
        }
        if index >= board[0].len() * board.len() {
            return false;
        }
        true
    }

    pub fn point_within_board(board: &[Vec<i8>], point: Point) -> bool {
        if board.is_empty() || board[0].is_empty() {
            return false;
        }
        if point.x >= board[0].len() || point.y >= board.len() {
            return false;
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_get_offset_dimension() {
        let new_val = Point::get_offset_dimension(10, 2, 1);
        assert_eq!(new_val, Some(3));
        let new_val = Point::get_offset_dimension(10, 2, -1);
        assert_eq!(new_val, Some(1));
        let new_val = Point::get_offset_dimension(10, 2, 3);
        assert_eq!(new_val, Some(5));
        let new_val = Point::get_offset_dimension(10, 2, -2);
        assert_eq!(new_val, Some(0));
        let new_val = Point::get_offset_dimension(10, 2, 8);
        assert_eq!(new_val, None);
        let new_val = Point::get_offset_dimension(10, 2, -3);
        assert_eq!(new_val, None);
        let new_val = Point::get_offset_dimension(10, 2, 10);
        assert_eq!(new_val, None);
        let new_val = Point::get_offset_dimension(10, 2, -5);
        assert_eq!(new_val, None);
    }

    #[test]
    fn test_point_get_offset() {
        let point = Point { x: 2, y: 2 };
        let new_point = point.get_offset(10, 10, 0, 0);
        assert_eq!(new_point, Some(Point { x: 2, y: 2 }));
        let new_point = point.get_offset(10, 10, -2, -2);
        assert_eq!(new_point, Some(Point { x: 0, y: 0 }));
        let new_point = point.get_offset(10, 10, 7, 7);
        assert_eq!(new_point, Some(Point { x: 9, y: 9 }));
        let new_point = point.get_offset(10, 10, 8, 7);
        assert_eq!(new_point, None);
        let new_point = point.get_offset(10, 10, 7, 8);
        assert_eq!(new_point, None);
        let new_point = point.get_offset(10, 10, -3, -3);
        assert_eq!(new_point, None);
    }

    #[test]
    fn test_point_get_abs_distance() {
        let point = Point { x: 2, y: 2 };
        let other = Point { x: 0, y: 8 };
        assert_eq!(point.get_abs_distance(&other), 8);
    }
}
