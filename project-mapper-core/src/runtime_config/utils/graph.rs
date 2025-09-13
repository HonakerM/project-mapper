use crate::runtime_config::{
    RuntimeConfig,
    shared::{ComponentConfig, Uid},
};
use anyhow::{Result, anyhow};
use petgraph::{acyclic::Acyclic, adj::Neighbors, visit::{Bfs, IntoNeighbors}, Direction};
use petgraph::data::Build;
use petgraph::graph::{Graph, NodeIndex};
use std::collections::{HashMap, HashSet};

pub type GraphType = Acyclic<Graph<Box<dyn ComponentConfig>, ()>>;

/// Stores an acyclic graph constructed from RuntimeConfig, and unused components.
#[derive(Debug)]
pub struct RuntimeConfigGraph {
    pub graph: GraphType,
    pub unused_nodes: Vec<Box<dyn ComponentConfig>>,
	pub root_nodes: Vec<NodeIndex>,	
	pub node_mapping: HashMap<Uid, NodeIndex>,
}

impl RuntimeConfigGraph {
    /// Constructs an acyclic graph from a RuntimeConfig.
    /// Uses ComponentConfig as nodes, dependents() for edges, and outputs as roots.
    pub fn new(config: &RuntimeConfig) -> Result<Self> {
        let mut graph: GraphType = Acyclic::new();
        let mut uid_to_index: HashMap<Uid, NodeIndex> = HashMap::new();
        let mut used_uids: HashSet<Uid> = HashSet::new();

        // Gather all configs
        let all_configs = config.gather_configs();
        let mut uid_to_config: HashMap<Uid, Box<dyn ComponentConfig>> = HashMap::new();
        for cfg in all_configs.into_iter() {
            uid_to_config.insert(cfg.uid(), cfg);
        }

        // Helper to insert node if not already present
        let mut insert_node =
            |uid: Uid, uid_to_index: &mut HashMap<Uid, NodeIndex>, graph: &mut GraphType| {
                if let Some(cfg) = uid_to_config.get(&uid) {
                    if !uid_to_index.contains_key(&uid) {
                        // Use Acyclic::add_node
                        let idx = graph.add_node(cfg.clone());
                        uid_to_index.insert(uid, idx);
                    }
                }
            };

        // Recursive DFS from outputs
        fn dfs(
            uid: Uid,
            uid_to_config: &HashMap<Uid, Box<dyn ComponentConfig>>,
            uid_to_index: &mut HashMap<Uid, NodeIndex>,
            graph: &mut GraphType,
            used_uids: &mut HashSet<Uid>,
            insert_node: &mut impl FnMut(Uid, &mut HashMap<Uid, NodeIndex>, &mut GraphType),
        ) -> Result<()> {
            if used_uids.contains(&uid) {
                return Ok(());
            }
            used_uids.insert(uid);
            insert_node(uid, uid_to_index, graph);

            let dependents = uid_to_config
                .get(&uid)
                .map(|cfg| cfg.dependents())
                .unwrap_or_default();
            for dep_uid in &dependents {
                insert_node(*dep_uid, uid_to_index, graph);
            }
            for dep_uid in dependents {
                if let (Some(&from), Some(&to)) =
                    (uid_to_index.get(&dep_uid), uid_to_index.get(&uid))
                {
                    if graph.try_add_edge(from, to, ()).is_err() {
                        return Err(anyhow!(
                            "Cycle detected when adding edge from {} to {}",
                            dep_uid,
                            uid
                        ));
                    }
                }
                dfs(
                    dep_uid,
                    uid_to_config,
                    uid_to_index,
                    graph,
                    used_uids,
                    insert_node,
                )?;
            }
            Ok(())
        }

        // Start DFS from each output
        for output in &config.outputs {
            let uid = output.uid();
            dfs(
                uid,
                &uid_to_config,
                &mut uid_to_index,
                &mut graph,
                &mut used_uids,
                &mut insert_node,
            )?;
        }

        // Find unused nodes
        let unused_nodes: Vec<Box<dyn ComponentConfig>> = uid_to_config
            .iter()
            .filter(|(uid, _)| !used_uids.contains(uid))
            .map(|(_, cfg)| cfg.clone())
            .collect();

	    let root_nodes: Vec<NodeIndex> = graph
	        .node_indices()
	        .filter(|&n| graph.neighbors_directed(n, Direction::Incoming).next().is_none())
	        .collect();

        Ok(Self {
            graph,
            unused_nodes,
			root_nodes,
			node_mapping: uid_to_index,
        })
    }


	pub fn bfs_traverse(&self) -> Vec<Box<dyn ComponentConfig>> {
		let mut visited = HashSet::new();
		let mut output_vec = Vec::new();
		for root in &self.root_nodes {
        	let mut bfs = Bfs::new(&self.graph, *root);
			while let Some(next_node) = bfs.next(&self.graph) {
				if visited.contains(&next_node) {continue}
				visited.insert(next_node);
				output_vec.push(self.graph[next_node].clone_box());
			}
		}
		output_vec
	}

	pub fn get_downstream_components(&self, uid: Uid)->HashSet<Box<dyn ComponentConfig>> {
		let node_index = self.node_mapping.get(&uid).unwrap();
		let mut neighbors = self.graph.neighbors_directed(*node_index, Direction::Incoming);
		let mut downstreams = HashSet::new();
		while let Some(neighbor) = neighbors.next(){
			downstreams.insert(
				self.graph.node_weight(neighbor).unwrap().clone()
			);
		}
		downstreams
	}
}
