use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    any::type_name_of_val,
    collections::{HashMap, HashSet},
    error::Error,
    fmt::{Debug, Display},
};

use crate::{
    runtime_config::{
        RuntimeConfig,
        effect::EffectComponentConfig,
        input::InputComponentConfig,
        output::OutputComponentConfig,
        shared::{ComponentConfig, Uid},
        utils::graph::RuntimeConfigGraph,
    },
    types::errors::RuntimeConfigValidationError,
};
use anyhow::{Result, anyhow};

#[derive(Debug)]
pub struct RuntimeConfigChangeTracker {
    pub graph: RuntimeConfigGraph,
    pub deletes: Vec<Uid>,
}

impl RuntimeConfigChangeTracker {
    pub fn gather_changes(org: &RuntimeConfig, new: &RuntimeConfig) -> Result<Self> {
        let org_comps = org.gather_configs();
        let org_comps: HashSet<&Box<dyn ComponentConfig>> = HashSet::from_iter(org_comps.iter());
        let new_comps = new.gather_configs();
        let new_comps: HashSet<&Box<dyn ComponentConfig>> = HashSet::from_iter(new_comps.iter());
        let deleted_comps = Vec::from_iter(org_comps.difference(&new_comps).map(|x| x.uid()));

        Ok(Self {
            graph: RuntimeConfigGraph::new(new)?,
            deletes: deleted_comps,
        })
    }
}
