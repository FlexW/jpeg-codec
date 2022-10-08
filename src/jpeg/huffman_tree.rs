use super::decoder::HuffmanTable;

pub struct HuffmanTree {
    nodes: Vec<HuffmanNode>,
}

struct HuffmanNode {
    value: u8,
    parent: Option<usize>,
    left_child: Option<usize>,
    right_child: Option<usize>,
    valid_code: bool,
}

impl HuffmanTree {
    pub fn new(huffman_table: &HuffmanTable) -> Self {
        let mut tree = Self { nodes: Vec::new() };
        tree.nodes.push(HuffmanNode::new()); // Root node
        tree.construct_tree(huffman_table);
        tree
    }

    pub fn print_codes(&self) {
        self.do_print_codes(0, String::new());
    }

    fn do_print_codes(&self, node_index: usize, code: String) {
        if node_index >= self.nodes.len() {
            return;
        }

        let node = &self.nodes[node_index];
        if let Some(left_child) = node.left_child {
            self.do_print_codes(left_child, code.clone() + "0");
        }

        if node.valid_code {
            println!("\t\tCode: {} Value: {}", code, node.value);
        }

        if let Some(right_child) = node.right_child {
            self.do_print_codes(right_child, code.clone() + "1");
        }
    }

    fn construct_tree(&mut self, huffman_table: &HuffmanTable) {
        // let &mut leftmost_node = Some(self.root_node.left_child);
        self.add_empty_childs(0);
        let mut leftmost_node = self.nodes[0].left_child;

        for i in 0..16 {
            if Self::symbol_count_of_length(huffman_table, i + 1) == 0 {
                let mut current = leftmost_node;
                while !current.is_none() {
                    self.add_empty_childs(current.unwrap());
                    current = self.get_right_node_on_same_level(current);
                }
                leftmost_node = self.nodes[leftmost_node.unwrap()].left_child;
            } else {
                for symbol in &huffman_table[i as usize] {
                    self.nodes[leftmost_node.unwrap()].value = *symbol;
                    self.nodes[leftmost_node.unwrap()].valid_code = true;

                    leftmost_node = self.get_right_node_on_same_level(leftmost_node);
                }

                self.add_empty_childs(leftmost_node.unwrap());
                let mut current = self.get_right_node_on_same_level(leftmost_node);
                leftmost_node = self.nodes[leftmost_node.unwrap()].left_child;

                while !current.is_none() {
                    self.add_empty_childs(current.unwrap());
                    current = self.get_right_node_on_same_level(current);
                }
            }
        }
    }

    fn add_empty_childs(&mut self, node_index: usize) {
        let left_node_index = self.nodes.len();
        self.nodes.push(HuffmanNode::new_with_parent(node_index));
        let right_node_index = self.nodes.len();
        self.nodes.push(HuffmanNode::new_with_parent(node_index));
        self.nodes[node_index].left_child = Some(left_node_index);
        self.nodes[node_index].right_child = Some(right_node_index);
    }

    fn symbol_count_of_length(huffman_table: &HuffmanTable, length: u8) -> u8 {
        huffman_table[(length - 1) as usize].len() as u8
    }

    fn get_right_node_on_same_level(&self, node_index: Option<usize>) -> Option<usize> {
        if node_index.is_none() {
            return None;
        }

        let node_index = node_index.unwrap();
        let node = &self.nodes[node_index];

        if let Some(parent) = node.parent {
            let parent_node = &self.nodes[parent];
            let is_parent_left_child = parent_node
                .left_child
                .map_or_else(|| false, |left_child| left_child == node_index);

            let parent_right_child = self.nodes[parent].right_child;

            if is_parent_left_child {
                return parent_right_child;
            }

            // Go back in tree until the current node is not anymore a right child node
            let mut current = node_index;
            let mut depth = 0;
            loop {
                if self.nodes[current].parent.is_none() {
                    return None;
                }
                if self.nodes[self.nodes[current].parent.unwrap()]
                    .right_child
                    .unwrap()
                    != current
                {
                    break;
                }
                current = self.nodes[current].parent.unwrap();
                depth += 1;
            }

            current = self.nodes[self.nodes[current].parent.unwrap()]
                .right_child
                .unwrap();

            while depth > 0 {
                current = self.nodes[current].left_child.unwrap();
                depth -= 1;
            }

            return Some(current);
        }
        return None;
    }
}

impl HuffmanNode {
    pub fn new() -> Self {
        Self {
            value: 0,
            parent: None,
            left_child: None,
            right_child: None,
            valid_code: false,
        }
    }

    pub fn new_with_parent(parent: usize) -> Self {
        Self {
            value: 0,
            parent: Some(parent),
            left_child: None,
            right_child: None,
            valid_code: false,
        }
    }
}
