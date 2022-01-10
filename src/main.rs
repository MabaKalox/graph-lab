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
    id: String,
}

pub struct GraphListElement {
    node: Box<GraphNode>,
    neighborhoods: List<Edge>,
}

impl GraphListElement {
    pub fn new(id: String, name: String) -> Self {
        Self {
            node: Box::new(GraphNode { id, name }),
            neighborhoods: List::new(),
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
    let mut graph: List<GraphListElement> = List::new();

    if let Err(err) = parse_csv_file(Path::new(NODES_FILE_PATH), |line| {
        let mut it = line.trim().split(",");
        match (it.next(), it.next()) {
            (Some(id), Some(name)) => {
                graph.push(GraphListElement::new(id.to_owned(), name.to_owned()));
                true
            }
            _ => false,
        }
    }) {
        return Err(err);
    }

    if let Err(err) = parse_csv_file(Path::new(EDGES_FILE_PATH), |line| {
        let mut it = line.trim().split(",");
        match (it.next(), it.next(), it.next()) {
            (Some(from), Some(to), Some(weight)) => {
                match graph.seek_mut_f(|node| from.eq(&node.node.id)).peek_mut() {
                    Some(v) => {
                        v.neighborhoods.push(Edge {
                            id: to.to_owned(),
                            weight: weight.parse::<i32>().map_or(None, |v| Some(v))
                        });
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }) {
        return Err(err);
    }

    for node in graph.iter() {
        print!("{}:", node.node.id);
        for edge in node.neighborhoods.iter() {
            print!(" -> {}", edge.id);
        }
        println!();
    }

    return Ok(());
}
