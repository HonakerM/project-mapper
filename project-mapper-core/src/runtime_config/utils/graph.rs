use crate::runtime_config::{shared::{ComponentConfig, Uid}, RuntimeConfig};
use petgraph::data::Build;
use petgraph::acyclic::Acyclic;
use petgraph::graph::{NodeIndex, Graph};
use std::collections::{HashMap, HashSet};
use anyhow::{Result, anyhow};


pub type GraphType =  Acyclic<Graph<Box<dyn ComponentConfig>, ()>>;

/// Stores an acyclic graph constructed from RuntimeConfig, and unused components.
pub struct RuntimeConfigGraph {
	pub graph: GraphType,
	pub unused_nodes: Vec<Box<dyn ComponentConfig>>,
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
		let mut insert_node = |uid: Uid, uid_to_index: &mut HashMap<Uid, NodeIndex>, graph: &mut GraphType| {
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
			insert_node: &mut impl FnMut(Uid, &mut HashMap<Uid, NodeIndex>,  &mut GraphType),
		) -> Result<()> {
			if used_uids.contains(&uid) { return Ok(()); }
			used_uids.insert(uid);
			insert_node(uid, uid_to_index, graph);

			let dependents = uid_to_config.get(&uid).map(|cfg| cfg.dependents()).unwrap_or_default();
			for dep_uid in &dependents {
				insert_node(*dep_uid, uid_to_index, graph);
			}
			for dep_uid in dependents {
				if let (Some(&from), Some(&to)) = (uid_to_index.get(&dep_uid), uid_to_index.get(&uid)) {
					if graph.try_add_edge(from, to, ()).is_err() {
						return Err(anyhow!("Cycle detected when adding edge from {} to {}", dep_uid, uid));
					}
				}
				dfs(dep_uid, uid_to_config, uid_to_index, graph, used_uids, insert_node)?;
			}
			Ok(())
		}

		// Start DFS from each output
		for output in &config.outputs {
			let uid = output.uid();
			dfs(uid, &uid_to_config, &mut uid_to_index, &mut graph, &mut used_uids, &mut insert_node)?;
		}

		// Find unused nodes
		let unused_nodes: Vec<Box<dyn ComponentConfig>> = uid_to_config
			.iter()
			.filter(|(uid, _)| !used_uids.contains(uid))
			.map(|(_, cfg)| cfg.clone())
			.collect();

		Ok(Self { graph, unused_nodes })
	}
}
