extern crate rand;

use rand::Rng;

use crate::astar::AStar;
use crate::common::Point;

pub struct LongestCycle<'a> {
    board: &'a [Vec<i8>],
    start: Point,
    target: Point,
    board_height: usize,
    board_width: usize,
}

impl LongestCycle<'_> {
    pub fn new(board: &[Vec<i8>], start: usize, target: usize) -> Option<LongestCycle> {
        if !Point::index_within_board(board, start) || !Point::index_within_board(board, target) {
            None
        } else {
            Some(LongestCycle {
                board,
                start: Point::get_point_from_index(board[0].len(), start),
                target: Point::get_point_from_index(board[0].len(), target),
                board_height: board.len(),
                board_width: board[0].len(),
            })
        }
    }

    pub fn get_longest_cycle(&self) -> Option<Vec<usize>> {
        for _ in 0..10 {
            if let Some(cycle) = self.try_get_longest_cycle() {
                if cycle.len() == self.board_width * self.board_height {
                    return Some(cycle);
                }
            }
        }
        None
    }

    fn try_get_longest_cycle(&self) -> Option<Vec<usize>> {
        if let Some(cycle) = LongestCycle::create_longest_cycle(self.board, self.start, self.target) {
            Some(
                cycle
                    .iter()
                    .map(|point| Point::get_index_from_point(self.board_width, point))
                    .collect(),
            )
        } else {
            None
        }
    }

    fn create_longest_cycle(board: &[Vec<i8>], start: Point, target: Point) -> Option<Vec<Point>> {
        if let Some(mut cycle) = LongestCycle::get_smallest_cycle(&board, start, target) {
            //println!("{:?}", cycle);
            let mut rng = rand::thread_rng();
            let mut extended = true;
            'extend: while extended {
                let start_index: usize = rng.gen_range(0, cycle.len());
                extended = false;
                let current_len = cycle.len();
                for i in 0..current_len {
                    extended = LongestCycle::extend_cycle(&board, &mut cycle, (start_index + i) % current_len);
                    if extended {
                        continue 'extend;
                    }
                }
            }
            //println!("Length: {:?}; {:?}", cycle.len(), cycle);
            Some(cycle)
        } else {
            None
        }
    }

    fn extend_cycle(board: &[Vec<i8>], cycle: &mut Vec<Point>, start_index: usize) -> bool {
        let board_height = board.len();
        let board_width = board[0].len();
        let current_len = cycle.len();
        let point_1 = cycle[start_index];
        let point_2 = cycle[(start_index + 1) % current_len];

        let mut test_points: Vec<Point> = Vec::new();
        if point_1.y == point_2.y {
            if let Some(this_point_1) = point_1.get_offset(board_width, board_height, 0, 1) {
                if let Some(this_point_2) = point_2.get_offset(board_width, board_height, 0, 1) {
                    test_points.push(this_point_1);
                    test_points.push(this_point_2);
                }
            }
            if let Some(this_point_1) = point_1.get_offset(board_width, board_height, 0, -1) {
                if let Some(this_point_2) = point_2.get_offset(board_width, board_height, 0, -1) {
                    test_points.push(this_point_1);
                    test_points.push(this_point_2);
                }
            }
        } else {
            if let Some(this_point_1) = point_1.get_offset(board_width, board_height, 1, 0) {
                if let Some(this_point_2) = point_2.get_offset(board_width, board_height, 1, 0) {
                    test_points.push(this_point_1);
                    test_points.push(this_point_2);
                }
            }
            if let Some(this_point_1) = point_1.get_offset(board_width, board_height, -1, 0) {
                if let Some(this_point_2) = point_2.get_offset(board_width, board_height, -1, 0) {
                    test_points.push(this_point_1);
                    test_points.push(this_point_2);
                }
            }
        }

        //println!("Point 1: {:?}; Point 2: {:?}; Test Points: {:?}", point_1, point_2, test_points);

        for points in test_points.chunks_exact(2) {
            if !cycle.iter().any(|&vertex| vertex == points[0]) && !cycle.iter().any(|&vertex| vertex == points[1]) {
                //println!("{:?}, {:?}: {:?}", start_index, ((start_index + 1) % current_len), points);
                cycle.splice(
                    ((start_index + 1) % current_len)..((start_index + 1) % current_len),
                    points.iter().cloned(),
                );
                return true;
            }
        }

        false
    }

    fn get_smallest_cycle(board: &[Vec<i8>], start: Point, target: Point) -> Option<Vec<Point>> {
        if !Point::point_within_board(board, start) || !Point::point_within_board(board, target) {
            return None;
        }
        //println!("{:?}, {:?}, {:?}", start, target, board);
        

        let mut board_wo_start_target = board.to_vec();
        board_wo_start_target[start.x][start.y] = 0;
        board_wo_start_target[target.x][target.y] = 0;
        //println!("{:?}, {:?}, {:?}", start, target, board_wo_start_target);

        let board_inverted: Vec<Vec<i8>> = board
            .iter()
            .map(|rows| rows.iter().map(|val| (val - 1).abs()).collect())
            .collect();
        //println!("{:?}, {:?}, {:?}", start, target, board_inverted);
            

        let mut path: Vec<Point>;

        if let Some(this_path) = AStar::astar(&board_wo_start_target, start, target) {
            path = this_path;
        } else {
            return None;
        }
        //println!("{:?}", path);

        if let Some(this_path) = AStar::astar(&board_inverted, target, start) {
            path.extend_from_slice(&this_path[1..this_path.len() - 1]);
        } else {
            return None;
        }
        //println!("{:?}", path);

        Some(path)
    }
}

pub fn demo() {
    let mut board = vec![vec![0_i8; 6]; 6];
    board[3][1] = 1;
    board[2][1] = 1;
    board[1][1] = 1;
    if let Some(longest) = LongestCycle::new(&board, 9, 7) {
        if let Some(cycle) = longest.get_longest_cycle() {
            println!("Length: {:?}; Cycle: {:?}", cycle.len(), cycle)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_longest_cycle_new() {
        let board = vec![vec![0_i8; 4]; 4];
        let longest = LongestCycle::new(&board, 0, 15);
        assert!(longest.is_some());
        let longest = LongestCycle::new(&board, 0, 16);
        assert!(longest.is_none());
    }

    #[test]
    fn test_longest_cycle_get_smallest_cycle() {
        let mut board = vec![vec![0_i8; 10]; 10];
        board[3][1] = 1;
        board[2][1] = 1;
        board[1][1] = 1;
        let start = Point { x: 3, y: 1 };
        let target = Point { x: 1, y: 1 };
        let path = LongestCycle::get_smallest_cycle(&board, start, target);
        assert!(path.is_some());
        assert_eq!(path.unwrap().len(), 6);
        let mut board = vec![vec![0_i8; 10]; 10];
        let start = Point { x: 1, y: 1 };
        let target = Point { x: 3, y: 5 };
        board[1][1] = 1;
        board[1][2] = 1;
        board[1][3] = 1;
        board[1][4] = 1;
        board[1][5] = 1;
        board[2][5] = 1;
        board[3][5] = 1;
        let path = LongestCycle::get_smallest_cycle(&board, start, target);
        println!("{:?}", path);
        assert!(path.is_some());
        assert_eq!(path.unwrap().len(), 12);
    }

    #[test]
    fn test_longest_cycle_extend_cycle() {
        let mut board = vec![vec![0_i8; 10]; 10];
        board[3][1] = 1;
        board[2][1] = 1;
        board[1][1] = 1;
        let start = Point { x: 3, y: 1 };
        let target = Point { x: 1, y: 1 };
        let mut cycle_maybe = LongestCycle::get_smallest_cycle(&board, start, target);
        assert!(cycle_maybe.is_some());
        let mut cycle = cycle_maybe.unwrap();
        assert_eq!(cycle.len(), 6);
        LongestCycle::extend_cycle(&board, &mut cycle, 2);
        assert_eq!(cycle.len(), 8);
        LongestCycle::extend_cycle(&board, &mut cycle, 6);
        assert_eq!(cycle.len(), 10);
    }

    #[test]
    fn test_longest_cycle_get_longest_cycle() {
        let mut board = vec![vec![0_i8; 6]; 6];
        board[3][1] = 1;
        board[2][1] = 1;
        board[1][1] = 1;
        let longest_maybe = LongestCycle::new(&board, 9, 7);
        assert!(longest_maybe.is_some());
        let longest = longest_maybe.unwrap();
        let cycle_maybe = longest.get_longest_cycle();
        assert!(cycle_maybe.is_some());
        let cycle = cycle_maybe.unwrap();
        assert_eq!(cycle.len(), 36);
    }
}
