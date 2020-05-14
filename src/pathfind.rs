use std::collections::HashMap;

use crate::constants::*;
use crate::game::Position;

#[derive(Copy, Clone)]
struct Node {
    index: usize,
    position: Position,
    g: i32,
    h: i32,
    f: i32,
}

impl Node {
    fn new(index: usize, position: Position, g: i32, h: i32, f: i32) -> Node {
        Node {
            index,
            position,
            g,
            h,
            f,
        }
    }
}

struct NodeList {
    length: usize,
    nodes: Vec<Node>,
    parents: Vec<usize>,
}

impl NodeList {
    fn new() -> NodeList {
        NodeList {
            length: 0,
            nodes: Vec::new(),
            parents: Vec::new(),
        }
    }

    fn push(&mut self, node: Node, parent: usize) {
        self.nodes.push(node);
        self.parents.push(parent);
        self.length += 1;
    }

    fn pop(&mut self) -> Option<(Node, usize)> {
        if self.length <= 0 {
            None
        } else {
            self.length -= 1;
            Some((self.nodes.pop().unwrap(), self.parents.pop().unwrap()))
        }
    }

    fn sort(&mut self) {
        let mut both: Vec<(&Node, &usize)> = self.nodes.iter().zip(self.parents.iter()).collect();
        both.sort_by(|(na, _pa), (nb, _pb)| nb.f.cmp(&na.f));
        //both.sort_by(|(na, pa), (nb, pb)| na.f.cmp(&nb.f));
        let (sorted_nodes, sorted_parents): (Vec<Node>, Vec<usize>) = both.iter().cloned().unzip();
        self.nodes.clear();
        self.parents.clear();
        for node in sorted_nodes {
            self.nodes.push(node);
        }
        for parent in sorted_parents {
            self.parents.push(parent);
        }
    }
}

fn astar(board: &Vec<Vec<i8>>, start: Position, end: Position) -> Option<Vec<Position>> {
    let mut node_index: usize = 1;
    let start_node = Node::new(node_index, start, 0, 0, 0);
    node_index += 1;
    let end_node = Node::new(node_index, end, 0, 0, 0);

    let mut open_list = NodeList::new();
    let mut closed_list = NodeList::new();

    open_list.push(start_node, 0);

    while open_list.length > 0 {
        // Find the least f and pop it off the open list
        open_list.sort();
        let (this_node, this_parent) = open_list.pop().unwrap();

        // Push this onto the closed list
        closed_list.push(this_node, this_parent);

        // If this is the end then end
        if this_node.position == end_node.position {
            let mut path_list: Vec<Position> = Vec::new();
            let mut current_parent = this_parent;

            // Push the last position
            path_list.push(this_node.position);
            'path: while current_parent > 0 {
                // Find parent position
                for i in 0..closed_list.length {
                    if closed_list.nodes[i].index == current_parent {
                        // Push the parents position
                        path_list.push(closed_list.nodes[i].position);
                        current_parent = closed_list.parents[i];
                        continue 'path;
                    }
                }
            }
            path_list.reverse();
            return Some(path_list);
        }

        // Generate children
        let mut child_list = NodeList::new();
        let offsets = [(1, 0), (0, 1), (-1, 0), (0, -1)];
        for (x, y) in offsets.iter() {
            let mut new_pos = this_node.position.clone();
            new_pos.offset(*x, *y);
            if new_pos == this_node.position || board[new_pos.x as usize][new_pos.y as usize] == 1 {
                continue;
            }

            node_index += 1;
            child_list.push(
                Node::new(node_index, new_pos, i32::MAX, i32::MAX, i32::MAX),
                this_node.index,
            );
        }

        // Loop through the children
        'child: while child_list.length > 0 {
            let (mut child_node, child_parent) = child_list.pop().unwrap();

            // Is the child in the closed list
            for i in 0..closed_list.length {
                if closed_list.nodes[i].position == child_node.position {
                    continue 'child;
                }
            }

            // Calculate f, g & h
            child_node.g = this_node.g + 1;
            child_node.h = (child_node.position.x as i32 - end_node.position.x as i32).abs()
                + (child_node.position.y as i32 - end_node.position.y as i32).abs();
            child_node.f = child_node.g + child_node.h;

            // Is the child in the open list
            for i in 0..open_list.length {
                if open_list.nodes[i].position == child_node.position && child_node.g > open_list.nodes[i].g {
                    continue 'child;
                }
            }

            // Add child to open list
            open_list.push(child_node, child_parent);
        }
    }

    None
}

pub struct AStar {
    start: Position,
    end: Position,
    board: Vec<Vec<i8>>,
    path: Vec<Position>,
}

impl AStar {
    pub fn new(start: Position, end: Position, board: Vec<Vec<i8>>) -> AStar {
        AStar {
            start,
            end,
            board,
            path: Vec::new(),
        }
    }

    pub fn calc_path(&mut self) {
        let path_list = astar(&self.board, self.start, self.end);
        if let Some(path) = path_list {
            self.path = path;
        }
    }

    pub fn get_path(&self) -> &Vec<Position> {
        &self.path
    }
}


pub struct Hamiltonian {
    graph: HashMap<usize, Vec<usize>>,
    width: usize, 
    height: usize,
    start: usize,
    path: Option<Vec<usize>>,
    calculated: bool,
}

impl Hamiltonian {

    pub fn new(width: usize, height: usize, start: usize) -> Hamiltonian {
        Hamiltonian {
            width, 
            height, 
            start,
            graph: Hamiltonian::create_graph(width, height),
            path: None,
            calculated: false,
        }
    }
    
    pub fn calc_hamiltonian_cycle(&mut self) -> Option<Vec<usize>> {
        if !self.calculated {
            let mut path: Vec<usize> = Vec::new();
            self.path = Hamiltonian::hamilton(&self.graph, self.graph.len(), self.start, &mut path);
        }
        self.path.clone()
    }
    
    fn create_graph(width: usize, height: usize) -> HashMap<usize, Vec<usize>> {

        let mut graph: HashMap<usize, Vec<usize>> = HashMap::new();
        
        let mut index: usize = 0;
        for i in 0..width {
            for j in 0..height {
                let mut this_list: Vec<usize> = Vec::new();
                if index % width != 0 {
                    this_list.push(index - 1);
                }
                if index % width != width - 1 {
                    this_list.push(index + 1);
                }
                if index >= width {
                    this_list.push(index - width);
                }
                if index < width * (height - 1) {
                    this_list.push(index + width);
                }
                graph.insert(index, this_list.clone());
                index += 1;
            }
        }
        
        graph

    }
    
    fn hamilton(
        graph: &HashMap<usize, Vec<usize>>,
        size: usize,
        point: usize,
        path: &mut Vec<usize>,
    ) -> Option<Vec<usize>> {

        //println!("Called Path: {:?}, Point: {:?}", &path, &point);
        if !path.iter().any(|&pt| pt == point) {
            path.push(point);

            if path.len() == size {
                if let Some(next_points) = graph.get(&point) {
                    if next_points.iter().any(|&pt| pt == path[0]) {
                        return Some(path.clone());
                    } else {
                        return None;
                    }
                }
            }

            if let Some(next_points) = graph.get(&point) {
                //println!("Next Points: {:?}", &next_points);

                for this_point in next_points.iter() {
                    let mut res_path = path.clone();
                    let candidate = Hamiltonian::hamilton(graph, size, *this_point, &mut res_path);
                    if candidate.is_some() {
                        return candidate;
                    }
                }
                //println!("Deadend: {:?}", &path);
            }
        } else {
            //println!("Point {:?} already in path: {:?}", &point, &path);
        }

        None
    }

}


fn longest() {

    let mut board = vec![vec![0; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize];
    //board[5][5] = 1;
    board[4][5] = 1;
    //board[3][5] = 1;
    
    let start = Position::new_xy(5, 5);
    let end = Position::new_xy(3, 5);
    let mut astar = AStar::new(start, end, board);
    astar.calc_path();
    println!("{:?}", astar.path);
    

}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nodelist_sort() {
        let mut nodes = NodeList::new();
        nodes.push(Node::new(0, Position::new_xy(0, 0), 0, 0, 1), 0);
        nodes.push(Node::new(1, Position::new_xy(0, 0), 0, 0, 2), 0);
        nodes.push(Node::new(2, Position::new_xy(0, 0), 0, 0, 3), 1);
        assert_eq!(nodes.nodes[0].f, 1);
        assert_eq!(nodes.parents[0], 0);
        nodes.sort();
        assert_eq!(nodes.nodes[0].f, 3);
        assert_eq!(nodes.parents[0], 1);
    }

    #[test]
    fn test_nodelist_pop() {
        let mut nodes = NodeList::new();
        nodes.push(Node::new(0, Position::new_xy(0, 0), 0, 0, 1), 0);
        nodes.push(Node::new(1, Position::new_xy(0, 0), 0, 0, 2), 0);
        nodes.push(Node::new(2, Position::new_xy(0, 0), 0, 0, 3), 1);
        assert_eq!(nodes.length, 3);
        let (node, parent) = nodes.pop().unwrap();
        assert_eq!(node.index, 2);
        assert_eq!(parent, 1);
        assert_eq!(nodes.length, 2);
        let (node, parent) = nodes.pop().unwrap();
        assert_eq!(node.index, 1);
        assert_eq!(parent, 0);
        assert_eq!(nodes.length, 1);
        let (node, parent) = nodes.pop().unwrap();
        assert_eq!(node.index, 0);
        assert_eq!(parent, 0);
        assert_eq!(nodes.length, 0);
        let nothing = nodes.pop();
        assert!(nothing.is_none());
    }

    #[test]
    fn test_astar() {
        let mut board = vec![vec![0; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize];
        board[6][6] = 1;
        board[6][5] = 1;
        board[5][4] = 1;

        let start = Position::new_xy(5, 5);
        let end = Position::new_xy(5, 2);
        let path_list = astar(&board, start, end);
        assert!(path_list.is_some());
        println!("{:?}", path_list);
        assert_eq!(path_list.unwrap().len(), 6);
    }

    #[test]
    fn test_astar_calc_path() {
        let board = vec![vec![0; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize];
        let start = Position::new_xy(5, 5);
        let end = Position::new_xy(7, 4);
        let mut astar = AStar::new(start, end, board);
        astar.calc_path();
        println!("{:?}", astar.path);
        assert_eq!(astar.path.len(), 4);
    }

    #[test]
    fn test_hamiltonian_new() {
        let mut ham = Hamiltonian::new(5, 6, 3);
        assert_eq!(ham.graph.len(), 30);
        assert_eq!(ham.graph.get(&0).unwrap().len(), 2);
        assert_eq!(ham.graph.get(&4).unwrap().len(), 2);
        assert_eq!(ham.graph.get(&25).unwrap().len(), 2);
        assert_eq!(ham.graph.get(&29).unwrap().len(), 2);
    }
    
    #[test]
    fn test_create_graph() {
        let mut ham = Hamiltonian::new(5, 6, 3);
        let path = ham.calc_hamiltonian_cycle();
        println!("{:?}", path);
        assert!(path.is_some());
        let this_path = path.unwrap();
        assert_eq!(this_path.len(), 5*6);
        assert_eq!(this_path[0], 3);
        let mut ham = Hamiltonian::new(3, 3, 0);
        let path = ham.calc_hamiltonian_cycle();
        println!("{:?}", path);
        assert!(path.is_none());
        let mut ham = Hamiltonian::new(2, 2, 0);
        let path = ham.calc_hamiltonian_cycle();
        println!("{:?}", path);
        assert!(path.is_some());
        let this_path = path.unwrap();
        assert_eq!(this_path.len(), 2*2);
        assert_eq!(this_path[0], 0);
    }
    
    #[test]
    fn test_hamiltonian_max() {
        let mut ham = Hamiltonian::new(6, 6, 0);
        let path = ham.calc_hamiltonian_cycle();
        println!("{:?}", path);
        assert!(path.is_some());
        assert_eq!(path.unwrap().len(), 6*6);
    }
    
    #[test]
    fn test_longest() {
        longest();
        assert!(false);
    }
    
}
