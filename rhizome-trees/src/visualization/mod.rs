use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use graphviz_rust::dot_structures;
use graphviz_rust::dot_structures::{Edge, GraphAttributes};

use graphviz_rust::printer::DotPrinter;
use uuid::Uuid;

pub struct TreeGraph {
    pub graph: dot_structures::Graph,
}

impl TreeGraph {
    pub fn new(label: String) -> TreeGraph {
        let mut g = TreeGraph {
            graph: dot_structures::Graph::DiGraph {
                id: dot_structures::Id::Anonymous(String::from("tree")),
                strict: false,
                stmts: vec![
                ],
            }
        };
        g.graph.add_stmt(dot_structures::Stmt::GAttribute(
            GraphAttributes::Graph(vec![
                dot_structures::Attribute(
                    dot_structures::Id::Plain(String::from("ordering")),
                    dot_structures::Id::Plain(String::from("out")),
                ),
                dot_structures::Attribute(
                    dot_structures::Id::Plain(String::from("label")),
                    dot_structures::Id::Escaped(format!("\"{}\"", label)),
                ),
            ])
        ));
        g
    }

    pub fn new_node(&mut self, label: String) -> dot_structures::NodeId {
        let id = dot_structures::NodeId(dot_structures::Id::Plain(format!("\"{:?}\"", Uuid::new_v4())), None);
        self.graph.add_stmt(dot_structures::Stmt::Node(
            dot_structures::Node::new(id.clone(), vec![
                dot_structures::Attribute(
                    dot_structures::Id::Plain(String::from("label")),
                    dot_structures::Id::Html(format!("{:?}", label)))
            ]),
        ));
        id
    }

    pub fn draw_edge(&mut self, from: &dot_structures::NodeId, to: &dot_structures::NodeId) {
        self.graph.add_stmt(dot_structures::Stmt::Edge(
            Edge {
                ty: dot_structures::EdgeTy::Pair(dot_structures::Vertex::N(from.clone()), dot_structures::Vertex::N(to.clone())),
                attributes: vec![],
            }
        ));
    }

    pub fn to_dot(&self) -> String {
        let mut ctx = graphviz_rust::printer::PrinterContext::default();
        self.graph.print(&mut ctx)
    }

    pub fn save_file(&self, filename: String) {
        let path = Path::new(&filename);
        let mut file = OpenOptions::new().create(true).write(true).truncate(true).open(path).expect("unable to open file");
        file.write_all(self.to_dot().as_bytes()).expect("unable to write output");
    }
}
