use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BTreeIndex {
    root: Option<Box<BTreeNode>>,
    order: usize, // max children per node
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct BTreeNode {
    keys: Vec<String>,
    values: Vec<usize>, // row ids
    children: Vec<Box<BTreeNode>>,
    is_leaf: bool,
}

impl BTreeNode {
    fn new(is_leaf: bool) -> Self {
        BTreeNode {
            keys: Vec::new(),
            values: Vec::new(),
            children: Vec::new(),
            is_leaf,
        }
    }

    fn is_full(&self, order: usize) -> bool {
        self.keys.len() >= order - 1
    }
}

impl BTreeIndex {
    pub fn new(order: usize) -> Self {
        BTreeIndex {
            root: Some(Box::new(BTreeNode::new(true))),
            order,
        }
    }

    pub fn insert(&mut self, key: String, row_id: usize) {
        let order = self.order;
        if let Some(ref mut root) = self.root {
            if root.is_full(order) {
                let mut new_root = Box::new(BTreeNode::new(false));
                let old_root = self.root.take().unwrap();
                new_root.children.push(old_root);
                Self::split_child(&mut new_root, 0, order);
                self.root = Some(new_root);
            }
        }

        if let Some(ref mut root) = self.root {
            Self::insert_non_full(order, root, key, row_id);
        }
    }

    fn insert_non_full(order: usize, node: &mut Box<BTreeNode>, key: String, row_id: usize) {
        let mut i = node.keys.len();

        if node.is_leaf {
            node.keys.push(key.clone());
            node.values.push(row_id);
            
            // sort to maintain order
            while i > 0 && node.keys[i] < node.keys[i - 1] {
                node.keys.swap(i, i - 1);
                node.values.swap(i, i - 1);
                i -= 1;
            }
        } else {
            while i > 0 && key < node.keys[i - 1] {
                i -= 1;
            }

            if node.children[i].is_full(order) {
                Self::split_child(node, i, order);
                if key > node.keys[i] {
                    i += 1;
                }
            }

            let child = &mut node.children[i];
            Self::insert_non_full(order, child, key, row_id);
        }
    }

    fn split_child(parent: &mut Box<BTreeNode>, index: usize, order: usize) {
        let full_child = &mut parent.children[index];
        let mut new_child = Box::new(BTreeNode::new(full_child.is_leaf));

        let mid = order / 2;

        new_child.keys = full_child.keys.split_off(mid);
        new_child.values = full_child.values.split_off(mid);

        if !full_child.is_leaf {
            new_child.children = full_child.children.split_off(mid + 1);
        }

        let promoted_key = full_child.keys.pop().unwrap();
        let promoted_value = full_child.values.pop().unwrap();

        parent.keys.insert(index, promoted_key);
        parent.values.insert(index, promoted_value);
        parent.children.insert(index + 1, new_child);
    }

    pub fn search(&self, key: &str) -> Option<usize> {
        self.root.as_ref().and_then(|root| self.search_node(root, key))
    }

    fn search_node(&self, node: &BTreeNode, key: &str) -> Option<usize> {
        let mut i = 0;
        while i < node.keys.len() && key > node.keys[i].as_str() {
            i += 1;
        }

        if i < node.keys.len() && key == node.keys[i].as_str() {
            return Some(node.values[i]);
        }

        if node.is_leaf {
            return None;
        }

        self.search_node(&node.children[i], key)
    }

    // range scan for queries like WHERE id > 5 AND id < 10
    pub fn range_scan(&self, start: &str, end: &str) -> Vec<usize> {
        let mut results = Vec::new();
        if let Some(ref root) = self.root {
            self.range_scan_node(root, start, end, &mut results);
        }
        results
    }

    fn range_scan_node(&self, node: &BTreeNode, start: &str, end: &str, results: &mut Vec<usize>) {
        let mut i = 0;

        while i < node.keys.len() {
            if !node.is_leaf {
                if node.keys[i].as_str() > start {
                    self.range_scan_node(&node.children[i], start, end, results);
                }
            }

            if node.keys[i].as_str() >= start && node.keys[i].as_str() <= end {
                results.push(node.values[i]);
            }

            i += 1;
        }

        if !node.is_leaf && i < node.children.len() {
            self.range_scan_node(&node.children[i], start, end, results);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_search() {
        let mut btree = BTreeIndex::new(3);
        btree.insert("5".to_string(), 0);
        btree.insert("3".to_string(), 1);
        btree.insert("7".to_string(), 2);

        assert_eq!(btree.search("5"), Some(0));
        assert_eq!(btree.search("3"), Some(1));
        assert_eq!(btree.search("7"), Some(2));
        assert_eq!(btree.search("10"), None);
    }

    #[test]
    fn test_range_scan() {
        let mut btree = BTreeIndex::new(3);
        btree.insert("1".to_string(), 0);
        btree.insert("5".to_string(), 1);
        btree.insert("10".to_string(), 2);
        btree.insert("15".to_string(), 3);

        let results = btree.range_scan("5", "15");
        assert_eq!(results.len(), 3);
    }
}