//! The `<LD>` graph's edges and their reduction to rungs (`plcopen_read.rs` has the reading's rules).

use std::collections::{HashMap, HashSet};

use roxmltree::Node;

use crate::plcopen_write::WIRE;

use super::{
    Edge, LEFT, Nodes, RIGHT, Tree, attr, iec_pin_name, kid, kids, links_of, par, ser, text_of,
    wire,
};

pub(super) enum Refused {
    /// An element a rung has no word for, named.
    Element(String),
    /// The body as a whole.
    Whole(String),
}

fn refused<T>(what: String) -> Result<T, Refused> {
    Err(Refused::Element(what))
}

/// What stands ahead of a network in the file and belongs to it.
struct Ahead {
    at: usize,
    label: Option<String>,
    comment: Option<String>,
}

pub(super) struct Reader<'a, 'i> {
    elements: Vec<Node<'a, 'i>>,
    by_id: HashMap<i64, Node<'a, 'i>>,
    nodes: Nodes,
    edges: Vec<Edge>,
    /// Producers a right rail takes: their nodes end a rung.
    ends: Vec<String>,
    ahead: Vec<Ahead>,
}

fn id_of(node: Node) -> i64 {
    attr(node, "localId")
        .and_then(|v| v.parse().ok())
        .unwrap_or(-1)
}

impl<'a, 'i> Reader<'a, 'i> {
    pub(super) fn new(ld: Node<'a, 'i>) -> Self {
        let elements: Vec<Node> = ld.children().filter(Node::is_element).collect();
        let by_id = elements.iter().map(|e| (id_of(*e), *e)).collect();
        Reader {
            elements,
            by_id,
            nodes: Nodes::default(),
            edges: Vec::new(),
            ends: Vec::new(),
            ahead: Vec::new(),
        }
    }

    fn name_of(&self, id: i64) -> Option<&str> {
        self.by_id.get(&id).map(|n| n.tag_name().name())
    }

    /// The node a link comes from: the left rail's own, or a producer's output pin.
    fn out_key(&self, (id, param): &(i64, String)) -> String {
        if self.name_of(*id) == Some("leftPowerRail") {
            LEFT.to_string()
        } else {
            format!("out:{id}:{param}")
        }
    }

    /// The node an element's input is: joined to every link into it, or to the left rail when nothing links into it.
    fn input(&mut self, id: i64, point: Option<Node>) -> String {
        let into = format!("in:{id}");
        let links = links_of(point);
        if links.is_empty() {
            self.nodes.join(&into, LEFT);
        }
        for link in &links {
            let from = self.out_key(link);
            self.nodes.join(&into, &from);
        }
        into
    }

    /// The elements that take block `id`'s output `param`, anywhere in the body.
    fn consumers_of(&self, id: i64, param: &str) -> Vec<Node<'a, 'i>> {
        (self.elements.iter())
            .filter(|el| {
                let pins = kids(kid(Some(**el), "inputVariables"), "variable")
                    .map(|v| kid(Some(v), "connectionPointIn"));
                (kids(Some(**el), "connectionPointIn").map(Some).chain(pins))
                    .any(|p| links_of(p).iter().any(|(r, f)| *r == id && f == param))
            })
            .copied()
            .collect()
    }

    pub(super) fn rungs(mut self) -> Result<String, Refused> {
        for el in &self.elements {
            let name = el.tag_name().name();
            if ["connector", "continuation", "actionBlock"].contains(&name) {
                return Err(Refused::Whole(format!(
                    "it draws a {name} in its ladder, which a rung here does not have yet"
                )));
            }
        }
        for (at, el) in self.elements.clone().into_iter().enumerate() {
            self.element(el, at)?;
        }
        let rungs = self.networks()?;
        Ok(self.with_heads(rungs))
    }

    fn element(&mut self, el: Node<'a, 'i>, at: usize) -> Result<(), Refused> {
        let id = id_of(el);
        match el.tag_name().name() {
            "comment" => {
                let text = text_of(kid(Some(el), "content"));
                if !text.is_empty() {
                    self.ahead.push(Ahead {
                        at,
                        label: None,
                        comment: Some(text),
                    });
                }
            }
            "label" => self.ahead.push(Ahead {
                at,
                label: Some(attr(el, "label").unwrap_or_default().to_string()),
                comment: None,
            }),
            "jump" | "return" => {
                // the end of its rung, like the right rail, with what it does as the rung's last word
                let into = self.input(id, kid(Some(el), "connectionPointIn"));
                let text = match el.tag_name().name() {
                    "jump" => format!(">>{}", attr(el, "label").unwrap_or_default()),
                    _ => "RETURN".to_string(),
                };
                self.edges.push(Edge {
                    from: into,
                    to: RIGHT.into(),
                    tree: Tree::El { text, at },
                });
            }
            "leftPowerRail" | "vendorElement" | "inVariable" | "documentation" | "addData" => {}
            "outVariable" => {
                // one fed by a contact or a coil copies the rung into a place: a coil is the word for that
                let links = links_of(kid(Some(el), "connectionPointIn"));
                if links.iter().any(|(r, _)| self.name_of(*r) != Some("block")) {
                    let place = text_of(kid(Some(el), "expression"));
                    return refused(format!(
                        "the rung wired straight into a variable, '{place}' (draw a coil)"
                    ));
                }
            }
            "rightPowerRail" => {
                for point in kids(Some(el), "connectionPointIn") {
                    for link in links_of(Some(point)) {
                        let end = self.out_key(&link);
                        self.ends.push(end);
                    }
                }
            }
            "contact" | "coil" => self.symbol(el, id, at)?,
            "block" => {
                let edge = self.block(el, id, at)?;
                self.edges.push(edge);
            }
            other => return refused(format!("a <{other}> element")),
        }
        Ok(())
    }

    fn symbol(&mut self, el: Node<'a, 'i>, id: i64, at: usize) -> Result<(), Refused> {
        let kind = el.tag_name().name();
        let variable = text_of(kid(Some(el), "variable"));
        if variable.is_empty() {
            return refused(format!("a {kind} with no variable"));
        }
        let negated = attr(el, "negated") == Some("true");
        let edge = attr(el, "edge").unwrap_or("none");
        let storage = attr(el, "storage").unwrap_or("none");
        let marks: Vec<&str> = [
            if negated { "/" } else { "" },
            match edge {
                "rising" => "P ",
                "falling" => "N ",
                _ => "",
            },
            match storage {
                "set" => "S ",
                "reset" => "R ",
                _ => "",
            },
        ]
        .into_iter()
        .filter(|m| !m.is_empty())
        .collect();
        if marks.len() > 1 || (kind == "contact" && storage != "none") {
            let what: Vec<String> = [
                negated.then(|| "negated".to_string()),
                (edge != "none").then(|| format!("{edge}-edge")),
                (storage != "none").then(|| storage.to_string()),
            ]
            .into_iter()
            .flatten()
            .collect();
            return refused(format!(
                "a {kind} on '{variable}' that is {}",
                what.join(" and ")
            ));
        }
        let mark = marks.first().copied().unwrap_or_default();
        let into = self.input(id, kid(Some(el), "connectionPointIn"));
        let text = if kind == "contact" && mark.is_empty() && variable.eq_ignore_ascii_case(WIRE) {
            String::new()
        } else if kind == "contact" {
            format!("|{mark}{variable}|")
        } else {
            format!("({mark}{variable})")
        };
        self.edges.push(Edge {
            from: into,
            to: format!("out:{id}:"),
            tree: Tree::El { text, at },
        });
        Ok(())
    }

    fn block(&mut self, el: Node<'a, 'i>, id: i64, at: usize) -> Result<Edge, Refused> {
        let block_type = attr(el, "typeName").unwrap_or_default().to_string();
        let instance = attr(el, "instanceName").filter(|i| !i.is_empty());
        let label = instance.map_or_else(|| format!("a {block_type} box"), |i| format!("'{i}'"));
        // TC6 requires every pin to carry its formalParameter, and the pin is known only by it
        let pin_name = |v: Node, side: &str| -> Result<String, Refused> {
            let raw = attr(v, "formalParameter").unwrap_or_default();
            if raw.is_empty() {
                return refused(format!(
                    "{label} with an {side} pin that has no name (an empty formalParameter)"
                ));
            }
            Ok(iec_pin_name(&block_type, raw))
        };
        let mut pins: Vec<String> = Vec::new();
        let mut power_in: Option<(String, Vec<(i64, String)>)> = None;
        for v in kids(kid(Some(el), "inputVariables"), "variable") {
            let name = pin_name(v, "input")?;
            if attr(v, "negated") == Some("true") || attr(v, "edge").is_some_and(|e| e != "none") {
                return refused(format!(
                    "{label} with its input {name} negated or edge-sensing"
                ));
            }
            let point = kid(Some(v), "connectionPointIn");
            if let Some(expression) = kid(point, "expression") {
                pins.push(format!("{name} := {}", text_of(Some(expression))));
                continue;
            }
            let links = links_of(point);
            if links.is_empty() {
                continue;
            }
            if let [(only, _)] = links[..]
                && self.name_of(only) == Some("inVariable")
            {
                let value = text_of(kid(self.by_id.get(&only).copied(), "expression"));
                pins.push(format!("{name} := {value}"));
                continue;
            }
            // two pins linked into the rung, even from the same place: a rung reaches a box by one pin
            if let Some((first, _)) = &power_in {
                return refused(format!(
                    "{label} on the rung by two inputs, {first} and {name}"
                ));
            }
            power_in = Some((name, links));
        }
        if let Some((_, links)) = &power_in {
            for link in links.clone() {
                let from = self.out_key(&link);
                self.nodes.join(&format!("in:{id}"), &from);
            }
        }
        let mut power_out: Option<(String, String)> = None;
        for v in kids(kid(Some(el), "outputVariables"), "variable") {
            // links name the pin as the file does, so consumers are found by its raw name
            let raw = attr(v, "formalParameter").unwrap_or_default().to_string();
            let name = pin_name(v, "output")?;
            let on_pin = kid(kid(Some(v), "connectionPointOut"), "expression");
            let consumers = self.consumers_of(id, &raw);
            let places: Vec<&Node> = consumers
                .iter()
                .filter(|c| c.tag_name().name() == "outVariable")
                .collect();
            let bound: Vec<String> = (on_pin.map(|e| text_of(Some(e))).into_iter())
                .chain(places.iter().map(|p| text_of(kid(Some(**p), "expression"))))
                .collect();
            if bound.len() > 1 {
                return refused(format!("{label}'s {name} bound to two places"));
            }
            if let Some(place) = bound.first() {
                pins.push(format!("{name} => {place}"));
            }
            if consumers.len() > places.len() {
                if !bound.is_empty() {
                    return refused(format!(
                        "{label}'s {name} both bound to a place and carried on along the rung"
                    ));
                }
                if let Some((first, _)) = &power_out {
                    return refused(format!(
                        "{label} on the rung by two outputs, {first} and {name}"
                    ));
                }
                power_out = Some((name, raw));
            }
        }
        let function = instance.is_none();
        let out_link = power_out
            .as_ref()
            .map_or(String::new(), |(_, raw)| raw.clone());
        let to = format!("out:{id}:{out_link}");
        let Some((in_name, _)) = power_in else {
            // a function whose result starts the rung, with no EN drawn: electrically an edge from the rail
            if let (true, Some((out, _))) = (function, &power_out)
                && !out.eq_ignore_ascii_case("ENO")
            {
                pins.push(out.clone());
                let text = format!("{block_type}({})", pins.join(", "));
                return Ok(Edge {
                    from: LEFT.into(),
                    to,
                    tree: Tree::El { text, at },
                });
            }
            return refused(format!("{label} that nothing on the rung feeds"));
        };
        if function && !in_name.eq_ignore_ascii_case("EN") {
            return refused(format!(
                "a {block_type} function fed by the rung on {in_name} rather than EN"
            ));
        }
        if let (true, Some((out, _))) = (function, &power_out)
            && !out.eq_ignore_ascii_case("ENO")
        {
            return refused(format!(
                "a {block_type} function carrying the rung on from {out} rather than ENO"
            ));
        }
        let text = match instance {
            None => format!("{block_type}({})", pins.join(", ")),
            Some(head) => {
                let all: Vec<String> = std::iter::once(in_name)
                    .chain(pins)
                    .chain(power_out.map(|(name, _)| name))
                    .collect();
                format!("{head}({})", all.join(", "))
            }
        };
        Ok(Edge {
            from: format!("in:{id}"),
            to,
            tree: Tree::El { text, at },
        })
    }

    /// The graph's networks, each reduced to one rung, in the file's order.
    fn networks(&mut self) -> Result<Vec<(usize, String)>, Refused> {
        let left = self.nodes.find(LEFT);
        let right = self.nodes.find(RIGHT);
        let mut graph: Vec<Edge> = (self.edges.clone().into_iter())
            .map(|e| Edge {
                from: self.nodes.find(&e.from),
                to: self.nodes.find(&e.to),
                tree: e.tree,
            })
            .collect();
        for end in self.ends.clone() {
            let from = self.nodes.find(&end);
            graph.push(Edge {
                from,
                to: right.clone(),
                tree: wire(),
            });
        }
        // a node nothing leaves ends a rung too: a coil with no right rail drawn, a box whose output is not used
        for e in graph.clone() {
            if e.to != right && !graph.iter().any(|f| f.from == e.to) {
                graph.push(Edge {
                    from: e.to,
                    to: right.clone(),
                    tree: wire(),
                });
            }
        }
        // one wire between the same two nodes is enough
        let mut wires = HashSet::new();
        graph.retain(|e| {
            !matches!(&e.tree, Tree::Ser(items) if items.is_empty())
                || wires.insert((e.from.clone(), e.to.clone()))
        });
        if graph
            .iter()
            .any(|e| e.from != left && !graph.iter().any(|f| f.to == e.from))
        {
            return Err(Refused::Whole(
                "an element of its ladder is not connected to the left rail".into(),
            ));
        }
        let mut rungs = Vec::new();
        for group in groups(&graph, &left, &right) {
            let tree = reduce(group, &left, &right)?;
            rungs.push((tree.at(), format!("{};", tree.print())));
        }
        rungs.sort_by_key(|(at, _)| *at);
        Ok(rungs)
    }

    /// The rungs as `.ld` text: a label or a comment belongs to the first network after it in the file; one after the
    /// last network labels an empty rung of its own, where a jump to the end lands.
    fn with_heads(&mut self, mut rungs: Vec<(usize, String)>) -> String {
        let mut heads: HashMap<usize, (Option<String>, Vec<String>)> = HashMap::new();
        self.ahead.sort_by_key(|a| a.at);
        for a in &self.ahead {
            let k = match rungs.iter().position(|(at, _)| *at > a.at) {
                Some(k) => k,
                None => {
                    rungs.push((usize::MAX, ";".into()));
                    rungs.len() - 1
                }
            };
            let head = heads.entry(k).or_default();
            if a.label.is_some() {
                head.0.clone_from(&a.label);
            }
            head.1.extend(a.comment.clone());
        }
        let mut out = String::new();
        for (k, (_, text)) in rungs.iter().enumerate() {
            let (label, comments) = heads.remove(&k).unwrap_or_default();
            for line in comments.iter().flat_map(|c| c.lines()) {
                out.push_str(&format!("(* {} *)\n", line.trim().replace("*)", "* )")));
            }
            if let Some(label) = label {
                out.push_str(&format!("{label}: "));
            }
            out.push_str(text);
            out.push('\n');
        }
        out
    }
}

/// The networks: what is connected, the rails apart.
fn groups(graph: &[Edge], left: &str, right: &str) -> Vec<Vec<Edge>> {
    let mut placed = vec![false; graph.len()];
    let mut out = Vec::new();
    for start in 0..graph.len() {
        if placed[start] {
            continue;
        }
        placed[start] = true;
        let (mut group, mut stack) = (Vec::new(), vec![start]);
        while let Some(i) = stack.pop() {
            group.push(graph[i].clone());
            let inner: Vec<&str> = [graph[i].from.as_str(), graph[i].to.as_str()]
                .into_iter()
                .filter(|n| *n != left && *n != right)
                .collect();
            for (j, f) in graph.iter().enumerate() {
                if !placed[j]
                    && (inner.contains(&f.from.as_str()) || inner.contains(&f.to.as_str()))
                {
                    placed[j] = true;
                    stack.push(j);
                }
            }
        }
        out.push(group);
    }
    out
}

/// A network reduced until one edge runs rail to rail.
fn reduce(mut edges: Vec<Edge>, left: &str, right: &str) -> Result<Tree, Refused> {
    loop {
        if let [only] = &edges[..]
            && only.from == left
            && only.to == right
        {
            return Ok(only.tree.clone());
        }
        // every series first, so that the edges between two nodes meet all at once: coils drawn beside a jump are
        // taken together, before the jump that follows them
        let joined = series(&mut edges, left, right);
        if !(branches(&mut edges) || joined) {
            let why = if edges.iter().any(|e| e.from == e.to) {
                "a network of its ladder joins a path's two ends with a bare wire, which this reader takes only drawn \
                 as a contact on TRUE"
            } else {
                "a network of its ladder is not series-parallel (a bridge), which a rung here cannot express"
            };
            return Err(Refused::Whole(why.into()));
        }
    }
}

/// Whether any node with one edge in and one out was joined, each into one edge.
fn series(edges: &mut Vec<Edge>, left: &str, right: &str) -> bool {
    let mut joined = false;
    while let Some(v) = (edges.iter()).map(|e| e.to.clone()).find(|v| {
        v != left
            && v != right
            && count(edges, |e| e.to == *v) == 1
            && count(edges, |e| e.from == *v) == 1
    }) {
        let into = edges.iter().position(|e| e.to == v).expect("one edge in");
        let first = edges.remove(into);
        let out = edges
            .iter()
            .position(|e| e.from == v)
            .expect("one edge out");
        let next = edges.remove(out);
        edges.push(Edge {
            from: first.from,
            to: next.to,
            tree: ser(first.tree, next.tree),
        });
        joined = true;
    }
    joined
}

/// Whether any edges between the same two nodes were taken as one branch.
fn branches(edges: &mut Vec<Edge>) -> bool {
    let mut merged = false;
    let ends: Vec<(String, String)> = (edges.iter())
        .filter(|e| e.from != e.to)
        .map(|e| (e.from.clone(), e.to.clone()))
        .collect();
    for (from, to) in ends {
        let (same, rest): (Vec<Edge>, Vec<Edge>) = std::mem::take(edges)
            .into_iter()
            .partition(|e| e.from == from && e.to == to);
        *edges = rest;
        if same.len() >= 2 {
            edges.push(Edge {
                from,
                to,
                tree: par(same.into_iter().map(|e| e.tree).collect()),
            });
            merged = true;
        } else {
            edges.extend(same);
        }
    }
    merged
}

fn count(edges: &[Edge], f: impl Fn(&Edge) -> bool) -> usize {
    edges.iter().filter(|e| f(e)).count()
}
