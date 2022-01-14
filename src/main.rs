use my_linked_list::List;
use my_reader::BufReader;
use std::fmt::Formatter;
use std::io::{Error, ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::{fmt, io};

const NODES_FILE_PATH: &str = "./datasets/asoiaf-book5-nodes.csv";
const EDGES_FILE_PATH: &str = "./datasets/asoiaf-book5-edges.csv";

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
    pub fn new_undirected(
        nodes_source_file: &Path,
        edges_source_file: &Path,
    ) -> Result<Self, Error> {
        let mut graph = Graph {
            nodes: List::new(),
            adjacency_list_repr: List::new(),
        };

        if let Err(err) = parse_csv_file(nodes_source_file, |line| {
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

        if let Err(err) = parse_csv_file(edges_source_file, |line| {
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

        return Ok(graph);
    }

    pub fn fmt_nodes(&self) -> String {
        let mut buff = String::new();
        let mut it = self.nodes.iter();
        if let Some(node) = it.next() {
            buff += &node.name;
            for node in it {
                buff += &format!(", {}", node.name);
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
            format!(
                "Enable to open file: {}",
                file_path.to_str().expect("Something wend wrong with path")
            ),
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

fn main() -> std::io::Result<()> {
    let mut graph1 =
        match Graph::new_undirected(Path::new(NODES_FILE_PATH), Path::new(EDGES_FILE_PATH)) {
            Ok(graph) => graph,
            Err(e) => return Err(e),
        };

    println!("Known characters: [ {} ]", graph1.fmt_nodes());

    match get_user_input("Task num? [1, 2, 3, 4]: ").as_str() {
        "1" => {
            println!("{}", graph1);
        }
        "2" => {
            let character_name = get_user_input("Character to delete: ");
            if let Some(t) = graph1
                .nodes
                .seek_f(|n| n.name.eq(&character_name))
                .peek()
                .map(|n| n.clone())
            {
                if get_user_input(&format!("Found: {}, delete? [Y/n] ", character_name)) == "Y" {
                    graph1.remove(t);
                }
                println!("{}", graph1);
                println!("Deleted char name: {}", character_name);
            } else {
                println!("Not found");
            }
        }
        "3" => {
            let graph2_nodes_source_file =
                PathBuf::from(get_user_input("Second graph nodes dataset path: ").as_str());
            let graph2_edges_source_file =
                PathBuf::from(get_user_input("Second graph edges dataset path: ").as_str());

            let graph2 = match Graph::new_undirected(
                graph2_nodes_source_file.as_path(),
                graph2_edges_source_file.as_path(),
            ) {
                Ok(graph) => graph,
                Err(e) => return Err(e),
            };

            let mut dead_characters = List::new();

            for character in graph2.nodes.iter() {
                if graph1
                    .nodes
                    .seek_f(|node| node.id.eq(&character.id))
                    .peek()
                    .is_none()
                {
                    dead_characters.push(character.name.clone());
                }
            }

            let mut new_characters = List::new();

            for character in graph1.nodes.iter() {
                if graph2
                    .nodes
                    .seek_f(|node| node.id.eq(&character.id))
                    .peek()
                    .is_none()
                {
                    new_characters.push(character.name.clone());
                }
            }

            println!("Who died: {}", dead_characters);
            println!("Who new: {}", new_characters);
        }
        "4" => {
            todo!()
        }
        _ => return Err(Error::new(ErrorKind::InvalidData, "Invalid task number.")),
    };

    return Ok(());
}
