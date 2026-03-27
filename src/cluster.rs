//! Cluster and file transfer operations.

use crate::abi::{call_host, raw};
use crate::types::SkillError;
use serde::{Serialize, Deserialize};

/// Cluster node information.
#[derive(Deserialize, Default)]
pub struct ClusterNodeInfo {
    #[serde(default)]
    pub node_id: String,
    #[serde(default)]
    pub role: String,
    #[serde(default)]
    pub skills: Vec<String>,
}

/// List known cluster nodes.
pub fn cluster_node_list() -> Result<(String, Vec<ClusterNodeInfo>), SkillError> {
    #[derive(Deserialize, Default)]
    struct Resp {
        #[serde(default)]
        current_node: String,
        #[serde(default)]
        nodes: Vec<ClusterNodeInfo>,
        #[serde(default)]
        error: String,
    }
    #[derive(Serialize)]
    struct Empty {}
    let resp: Resp = call_host(raw::cluster_node_list, &Empty {})?;
    if !resp.error.is_empty() { return Err(SkillError(resp.error)); }
    Ok((resp.current_node, resp.nodes))
}

/// Send a file to a remote cluster node.
pub fn file_transfer_send(target_node: &str, local_path: &str, remote_path: &str) -> Result<(), SkillError> {
    #[derive(Serialize)]
    struct Req { target_node: String, local_path: String, remote_path: String }
    #[derive(Deserialize, Default)]
    struct Resp { #[serde(default)] error: String }
    let resp: Resp = call_host(raw::file_transfer_send, &Req {
        target_node: target_node.into(), local_path: local_path.into(), remote_path: remote_path.into(),
    })?;
    if !resp.error.is_empty() { return Err(SkillError(resp.error)); }
    Ok(())
}

/// Request a file from a remote cluster node (pull mode).
pub fn file_transfer_recv(source_node: &str, remote_path: &str, local_path: &str) -> Result<(), SkillError> {
    #[derive(Serialize)]
    struct Req { source_node: String, remote_path: String, local_path: String }
    #[derive(Deserialize, Default)]
    struct Resp { #[serde(default)] error: String }
    let resp: Resp = call_host(raw::file_transfer_recv, &Req {
        source_node: source_node.into(), remote_path: remote_path.into(), local_path: local_path.into(),
    })?;
    if !resp.error.is_empty() { return Err(SkillError(resp.error)); }
    Ok(())
}
