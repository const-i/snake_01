use std::cmp::Ordering;
use std::collections::BinaryHeap;

use crate::common::Point;

#[derive(Copy, Clone)]
struct Node {
    index: usize,
    parent: Option<usize>,
    point: Point,
    g: i32,
    h: i32,
    f: i32,
}

impl Ord for Node {
    fn cmp(&self, other: &Node) -> Ordering {
        // Notice that the we flip the ordering on costs.
        // In case of a tie we compare positions - this step is necessary
        // to make implementations of `PartialEq` and `Ord` consistent.
        // https://doc.rust-lang.org/std/collections/binary_heap/
        other.f.cmp(&self.f).then_with(|| self.index.cmp(&other.index))
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Node) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

impl Eq for Node {}

impl PartialEq for Node {
    fn eq(&self, other: &Node) -> bool {
        self.f == other.f
    }
}

impl Node {
    fn new(index: usize, parent: Option<usize>, point: Point) -> Node {
        Node {
            index,
            parent,
            point,
            g: 0,
            h: 0,
            f: 0,
        }
    }
}

pub struct AStar<'a> {
    board: &'a [Vec<i8>],
    start: Point,
    target: Point,
    board_height: usize,
    board_width: usize,
}

impl AStar<'_> {
    pub fn new(board: &[Vec<i8>], start: usize, target: usize) -> Option<AStar> {
        if board.is_empty() || board[0].is_empty() {
            return None;
        }
        let board_height = board.len();
        let board_width = board[0].len();
        if start >= board_height * board_width || target >= board_height * board_width {
            return None;
        }
        Some(AStar {
            board,
            start: Point::get_point_from_index(board_width, start),
            target: Point::get_point_from_index(board_width, target),
            board_height,
            board_width,
        })
    }

    pub fn calculate_path(&self) -> Option<Vec<usize>> {
        if let Some(path) = AStar::astar(self.board, self.start, self.target) {
            Some(
                path.iter()
                    .map(|point| Point::get_index_from_point(self.board_width, point))
                    .collect(),
            )
        } else {
            None
        }
    }

    pub fn astar(board: &[Vec<i8>], start: Point, target: Point) -> Option<Vec<Point>> {
        if !Point::point_within_board(board, start) || !Point::point_within_board(board, target) {
            return None;
        }
        let board_height = board.len();
        let board_width = board[0].len();
        
        //println!("{:?}, {:?}, {:?}", start, target, board);

        let mut node_index: usize = 0;

        node_index += 1;
        let start_node = Node::new(node_index, None, start);
        node_index += 1;
        let target_node = Node::new(node_index, None, target);

        let mut open_list: BinaryHeap<Node> = BinaryHeap::new();
        let mut closed_list: BinaryHeap<Node> = BinaryHeap::new();

        open_list.push(start_node);

        while !open_list.is_empty() {
            // Pop off the least f
            let this_node = open_list.pop().unwrap();

            // Push this node to the closed list
            closed_list.push(this_node);

            // Check if we reached the target
            if this_node.point == target_node.point {
                let mut path_list: Vec<Point> = Vec::new();
                let mut current_parent = this_node.parent;
                path_list.push(this_node.point);

                'path: while current_parent.is_some() {
                    for node in closed_list.iter() {
                        if node.index == current_parent.unwrap() {
                            path_list.push(node.point);
                            current_parent = node.parent;
                            continue 'path;
                        }
                    }
                }

                path_list.reverse();
                return Some(path_list);
            }

            // Generate children
            let mut child_list: Vec<Node> = Vec::new();
            let offsets = [(1, 0), (0, 1), (-1, 0), (0, -1)];
            for &(x, y) in offsets.iter() {
                if let Some(new_point) = this_node.point.get_offset(board_width, board_height, x, y) {
                    if board[new_point.x][new_point.y] == 1 {
                        continue;
                    }
                    node_index += 1;
                    child_list.push(Node::new(node_index, Some(this_node.index), new_point));
                }
            }

            // Loop through children
            'child: while !child_list.is_empty() {
                let mut child_node = child_list.pop().unwrap();

                // Skip if its already closed
                for node in closed_list.iter() {
                    if child_node.point == node.point {
                        continue 'child;
                    }
                }

                // Calculate f, g, & h
                child_node.g = this_node.g + 1;
                child_node.h = child_node.point.get_abs_distance(&this_node.point);
                child_node.f = child_node.g + child_node.h;

                // Check if we have a better g value already in the open list
                for node in open_list.iter() {
                    if child_node.point == node.point && child_node.g > node.g {
                        continue 'child;
                    }
                }

                // Add child to open list
                open_list.push(child_node);
            }
        }
        None
    }
}

pub fn demo() {
    let mut board = vec![vec![0_i8; 4]; 4];
    board[1][0] = 1;
    let start = 0;
    let target = 15;
    if let Some(astar) = AStar::new(&board, start, target) {
        let path = astar.calculate_path();
        println!("Path: {:?}", path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_astar_new() {
        let board = vec![vec![0_i8; 4]; 4];
        let astar = AStar::new(&board, 0, 15);
        assert!(astar.is_some());
        let astar = AStar::new(&board, 0, 16);
        assert!(astar.is_none());
    }

    #[test]
    fn test_astar_calculate_path() {
        let board = vec![vec![0_i8; 4]; 4];
        let astar_maybe = AStar::new(&board, 0, 15);
        assert!(astar_maybe.is_some());
        let astar = astar_maybe.unwrap();
        let path_maybe = astar.calculate_path();
        assert!(path_maybe.is_some());
        let path = path_maybe.unwrap();
        assert_eq!(path.len(), 7);
        let mut board = vec![vec![0_i8; 4]; 4];
        board[1][0] = 1;
        board[0][1] = 1;
        let astar_maybe = AStar::new(&board, 0, 15);
        assert!(astar_maybe.is_some());
        let astar = astar_maybe.unwrap();
        let path_maybe = astar.calculate_path();
        assert!(path_maybe.is_none());
    }
}
