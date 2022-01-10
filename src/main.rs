use my_linked_list::List;
use my_reader::BufReader;
use std::io;
use std::io::{Error, ErrorKind};
use std::path::Path;
use std::rc::Rc;

const NODES_FILE_PATH: &str = "asoiaf-book5-nodes.csv";
const EDGES_FILE_PATH: &str = "asoiaf-book5-edges.csv";

pub struct GraphNode {
    name: String,
    id: String,
}

pub struct Edge {
    weight: Option<i32>,
    node_prt: Rc<GraphNode>,
}

pub struct GraphListElement {
    node_ptr: Rc<GraphNode>,
    neighborhoods: List<Edge>,
}

impl GraphListElement {
    pub fn new(node_ptr: Rc<GraphNode>) -> Self {
        Self {
            node_ptr,
            neighborhoods: List::new(),
        }
    }
}

pub struct Graph {
    nodes: List<Rc<GraphNode>>,
    adjacency_list_repr: List<GraphListElement>,
}

impl Graph {
    pub fn new() -> Self {
        Graph {
            nodes: List::new(),
            adjacency_list_repr: List::new(),
        }
    }
}

pub fn parse_csv_file<F: FnMut(Rc<String>) -> bool>(
    file_path: &Path,
    mut handler: F,
) -> io::Result<()> {
    let mut line_counter = 1;
    if let Ok(mut lines) = BufReader::open(file_path) {
        lines.next();
        for line in lines {
            line_counter += 1;
            if let Ok(ln) = line {
                if !handler(ln) {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        format!(
                            "Enable to parse line: {}, in file: {}",
                            line_counter, NODES_FILE_PATH
                        ),
                    ));
                }
            } else {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!(
                        "Enable to read line: {} in file: {}",
                        line_counter,
                        file_path.to_str().unwrap()
                    ),
                ));
            }
        }
    } else {
        return Err(Error::new(
            ErrorKind::NotFound,
            format!("Enable to open file: {}", NODES_FILE_PATH),
        ));
    }
    return Ok(());
}

fn main() -> std::io::Result<()> {
    let mut graph = Graph::new();

    if let Err(err) = parse_csv_file(Path::new(NODES_FILE_PATH), |line| {
        let mut it = line.trim().split(",");
        match (it.next(), it.next()) {
            (Some(id), Some(name)) => {
                graph.nodes.push(Rc::new(GraphNode {
                    id: id.to_owned(),
                    name: name.to_owned(),
                }));
                true
            }
            _ => false,
        }
    }) {
        return Err(err);
    }
    for node in graph.nodes.iter() {
        graph.adjacency_list_repr.push(GraphListElement::new(node.clone()));
    }

    if let Err(err) = parse_csv_file(Path::new(EDGES_FILE_PATH), |line| {
        let mut it = line.trim().split(",");
        match (it.next(), it.next(), it.next()) {
            (Some(from), Some(to), Some(weight)) => {
                let mut combinations = List::new();
                combinations.push((from, to));
                combinations.push((to, from));
                for (x, y) in combinations.iter() {
                    if let Some(v) = graph.adjacency_list_repr.seek_mut_f(|node| x.eq(&node.node_ptr.id)).peek_mut() {
                        v.neighborhoods.push(Edge {
                            node_prt: graph.nodes.seek_f(|node| node.id.eq(y)).peek().unwrap().clone(),
                            weight: weight.parse::<i32>().map_or(None, |v| Some(v)),
                        });
                    } else {
                        return false;
                    }
                }
                return true;
            }
            _ => false,
        }
    }) {
        return Err(err);
    }

    for node in graph.adjacency_list_repr.iter() {
        print!("{}:", node.node_ptr.id);
        for edge in node.neighborhoods.iter() {
            print!(" -> {}", edge.node_prt.id);
        }
        println!();
    }

    return Ok(());
}
