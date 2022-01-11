use my_linked_list::List;
use my_reader::BufReader;
use std::fmt::Formatter;
use std::io::{Error, ErrorKind, Write};
use std::path::Path;
use std::rc::Rc;
use std::{fmt, io};

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

    pub fn fmt_nodes(&self) -> String {
        let mut buff = String::new();
        let mut it = self.nodes.iter();
        if let Some(node) = it.next() {
            buff += &node.id;
            for node in it {
                buff += &format!(", {}", node.id);
            }
        }
        return buff;
    }

    pub fn remove(&mut self, t: Rc<GraphNode>) -> Rc<GraphNode> {
        self.adjacency_list_repr
            .remove_f(|n| n.node_ptr.id.eq(&t.id))
            .expect("Something went wrong at node deleting");
        for node in self.adjacency_list_repr.iter_mut() {
            node.neighborhoods.remove_f(|n| n.node_prt.id.eq(&t.id));
        }
        self.nodes
            .remove_f(|n| n.id.eq(&t.id))
            .expect("Something went wrong at node deleting")
    }
}

impl fmt::Display for Graph {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut buff = String::new();
        for node in self.adjacency_list_repr.iter() {
            buff += &format!("{}:", node.node_ptr.id);
            for edge in node.neighborhoods.iter() {
                buff += &format!(" -> {}", edge.node_prt.id);
            }
            buff += "\n";
        }
        write!(f, "{}", buff)
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

pub fn get_user_input(prompt_msg: &str) -> String {
    let mut input = String::new();
    print!("{}", prompt_msg);
    io::stdout().flush().expect("Could not flush buffer");
    io::stdin()
        .read_line(&mut input)
        .expect("Couldn't read line");
    return input.trim().to_owned();
}

pub fn del_demo(graph: &mut Graph) {
    let input = get_user_input("Character to delete: ");
    if let Some(t) = graph
        .nodes
        .seek_f(|n| n.id.eq(&input))
        .peek()
        .map(|n| n.clone())
    {
        if get_user_input(&format!("Found: {}, delete? [Y/n] ", input)) == "Y" {
            graph.remove(t);
        }
    } else {
        println!("Not found");
    }
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
    for node_ptr in graph.nodes.iter() {
        graph
            .adjacency_list_repr
            .push(GraphListElement::new(node_ptr.clone()));
    }

    if let Err(err) = parse_csv_file(Path::new(EDGES_FILE_PATH), |line| {
        let mut it = line.trim().split(",");
        match (it.next(), it.next(), it.next()) {
            (Some(from), Some(to), Some(weight)) => {
                let mut combinations = List::new();
                combinations.push((from, to));
                combinations.push((to, from));
                for (x, y) in combinations.iter() {
                    if let Some(v) = graph
                        .adjacency_list_repr
                        .seek_mut_f(|node| x.eq(&node.node_ptr.id))
                        .peek_mut()
                    {
                        v.neighborhoods.push(Edge {
                            node_prt: graph
                                .nodes
                                .seek_f(|node| node.id.eq(y))
                                .peek()
                                .unwrap()
                                .clone(),
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

    println!("Known characters: [ {} ]", graph.fmt_nodes());

    del_demo(&mut graph);

    println!("{}", graph);

    return Ok(());
}
