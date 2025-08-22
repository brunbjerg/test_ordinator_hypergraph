use std::collections::HashMap;
use std::collections::hash_map::Entry;

use chrono::NaiveDate;
use tracing::debug;

use crate::work_order::ActivityNumber;
use crate::work_order::ActivityRelation;
use crate::work_order::WorkOrder;
use crate::work_order::WorkOrderNumber;

#[derive(Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub enum ScheduleGraphErrors
{
    WorkerMissing,
    WorkOrderMissing,
    WorkOrderDuplicate,
    WorkOrderActivityMissingSkills,
    PeriodMissing,
    PeriodDuplicate,
    SkillMissing,
}

pub type NodeIndex = usize;
pub type EdgeIndex = usize;

pub type Worker = usize;

#[derive(Hash, Copy, Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub struct Period(NaiveDate);

#[derive(Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub struct HyperEdge
{
    edge_type: EdgeType,
    nodes: Vec<NodeIndex>,
}

#[derive(Hash, Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub enum Nodes
{
    Worker(Worker),
    WorkOrder(WorkOrderNumber),
    Activity(ActivityNumber),
    Period(Period),
    Skill(Skills),
    Day(NaiveDate),
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub enum Skills
{
    MtnMech,
    MtnElec,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub enum EdgeType
{
    Assign,
    Available,
    Contains,
    Requires,
    StartStart,
    FinishStart,
}

#[derive(Debug)]
pub struct ScheduleGraph
{
    /// Nodes of the problem
    nodes: Vec<Nodes>,

    /// Hyperedges to handle all the complex interactions
    hyperedges: Vec<HyperEdge>,

    /// Adjacency matrix
    adjacency: Vec<Vec<EdgeIndex>>,

    /// Indices to look up nodes
    worker_indices: HashMap<Worker, NodeIndex>,
    work_order_indices: HashMap<WorkOrderNumber, NodeIndex>,
    period_indices: HashMap<Period, NodeIndex>,
    skill_indices: HashMap<Skills, NodeIndex>,
    day_indices: HashMap<NaiveDate, NodeIndex>,
}

/// Public methods
impl ScheduleGraph
{
    pub fn new() -> Self
    {
        Self {
            nodes: vec![],
            hyperedges: vec![],
            adjacency: vec![],
            worker_indices: HashMap::new(),
            work_order_indices: HashMap::new(),
            period_indices: HashMap::new(),
            skill_indices: HashMap::new(),
            day_indices: HashMap::new(),
        }
    }
}

/// Public API to add [`Nodes`] to the graph.
impl ScheduleGraph
{
    pub fn add_work_order(&mut self, work_order: &WorkOrder) -> Result<NodeIndex, ScheduleGraphErrors>
    {
        let work_order_node = match self.work_order_indices.entry(work_order.number()) {
            Entry::Vacant(_new_work_order) => self.add_node(Nodes::WorkOrder(work_order.number())),
            Entry::Occupied(_already_inserted_work_order) => return Err(ScheduleGraphErrors::WorkOrderDuplicate),
        };

        if !work_order
            .activities()
            .iter()
            .all(|activity| self.skill_indices.keys().any(|&all_skills| all_skills == activity.skill()))
        {
            return Err(ScheduleGraphErrors::WorkOrderActivityMissingSkills);
        }

        //
        let mut previous_activity_node = usize::MAX;
        let activity_relations = work_order.activities_relations();
        for (activity_index, activity) in work_order.activities().iter().enumerate() {
            let activity_node = self.add_node(Nodes::Activity(activity.number()));
            let skill_node = *self.skill_indices.get(&activity.skill()).ok_or(ScheduleGraphErrors::SkillMissing)?;

            self.add_edge(EdgeType::Contains, vec![work_order_node, activity_node]);
            self.add_edge(EdgeType::Requires, vec![activity_node, skill_node]);

            if activity_index != 0 {
                match activity_relations[activity_index - 1] {
                    ActivityRelation::StartStart => self.add_edge(EdgeType::StartStart, vec![previous_activity_node, activity_node]),
                    ActivityRelation::FinishStart => self.add_edge(EdgeType::FinishStart, vec![previous_activity_node, activity_node]),
                    ActivityRelation::Postpone(_time_delta) => todo!(),
                };
            };
            previous_activity_node = activity_node;
        }

        // TODO [ ] - add relationships between activities here.

        self.work_order_indices.insert(work_order.number(), work_order_node);
        Ok(work_order_node)
    }

    pub fn add_period(&mut self, period: Period) -> Result<NodeIndex, ScheduleGraphErrors>
    {
        if self.period_indices.contains_key(&period) {
            return Err(ScheduleGraphErrors::PeriodDuplicate);
        };

        let days_in_period = (0..14).map(|e| period.0 + chrono::Days::new(e)).collect::<Vec<_>>();

        for day in days_in_period {
            let day_node = self.add_node(Nodes::Day(day));
            self.day_indices.insert(day, day_node);
        }

        let node_id = self.add_node(Nodes::Period(period));

        self.period_indices.insert(period, node_id);
        Ok(node_id)
    }
}

/// Public API to add [`HyperEdges`] to the graph
impl ScheduleGraph
{
    // TODO [ ] - this should be formulated as ids... it should be the types that
    // are found inside of the `Nodes` enum variants.
    pub fn add_assignment(&mut self, worker: Worker, work_order: WorkOrderNumber, date: Period) -> Result<EdgeIndex, ScheduleGraphErrors>
    {
        // This should return an error if the `Nodes` is not present.
        let worker = self.worker_indices.get(&worker).ok_or(ScheduleGraphErrors::WorkerMissing)?;
        let work_order = self.work_order_indices.get(&work_order).ok_or(ScheduleGraphErrors::WorkOrderMissing)?;
        let date = self.period_indices.get(&date).ok_or(ScheduleGraphErrors::PeriodMissing)?;

        let hyperedge = HyperEdge {
            edge_type: EdgeType::Assign,
            nodes: vec![*worker, *work_order, *date],
        };

        self.hyperedges.push(hyperedge);
        Ok(self.hyperedges.len() - 1)
    }

    pub fn find_all_assignments_for_period(&self, period_start_date: Period) -> Result<Vec<HyperEdge>, ScheduleGraphErrors>
    {
        if !self.nodes.iter().any(|e| (e == &Nodes::Period(period_start_date))) {
            return Err(ScheduleGraphErrors::PeriodMissing);
        }
        Ok(self
            .hyperedges
            .iter()
            .filter(|e| matches!(e.edge_type, EdgeType::Assign))
            .cloned()
            .collect())
    }

    pub fn add_assign_skill_to_worker(&mut self, worker: Worker, skill: Skills) -> Result<EdgeIndex, ScheduleGraphErrors>
    {
        let worker = self.worker_indices.get(&worker).ok_or(ScheduleGraphErrors::WorkerMissing)?;
        let skill = self.skill_indices.get(&skill).ok_or(ScheduleGraphErrors::SkillMissing)?;

        Ok(self.add_edge(EdgeType::Assign, vec![*worker, *skill]))
    }
}

/// Private methods.
///
/// [`NodeIndex`] and [`EdgeIndex`] are not allowed to be a part of the
/// public API of the type. The graph should only expose domain types
/// found in `ordinator-scheduling-environment`
impl ScheduleGraph
{
    fn add_node(&mut self, node: Nodes) -> NodeIndex
    {
        // This is the next element as `len()` is one larger than the last index
        let node_index = self.nodes.len();
        let none_checker = match node {
            Nodes::Worker(worker) => self.worker_indices.insert(worker, node_index),
            Nodes::WorkOrder(work_order) => self.work_order_indices.insert(work_order, node_index),
            Nodes::Period(naive_date) => self.period_indices.insert(naive_date, node_index),
            Nodes::Skill(skills) => self.skill_indices.insert(skills, node_index),
            Nodes::Activity(a) => {
                debug!(target: "developer", activity = a, "No node index for `Activities`");
                None
            }
            Nodes::Day(naive_date) => self.day_indices.insert(naive_date, node_index),
        };
        assert!(none_checker.is_none());

        self.adjacency.push(vec![]);

        // node is added `Vec<Nodes>`
        self.nodes.push(node);
        node_index
    }

    fn add_edge(&mut self, edge_type: EdgeType, nodes: Vec<NodeIndex>) -> EdgeIndex
    {
        let edge_index = self.hyperedges.len();

        for node_index in &nodes {
            self.adjacency[*node_index].push(edge_index);
        }
        let hyper_edge = HyperEdge { edge_type, nodes };
        self.hyperedges.push(hyper_edge);
        edge_index
    }
}
impl Default for ScheduleGraph
{
    fn default() -> Self
    {
        Self::new()
    }
}

#[cfg(test)]
mod tests
{
    use std::collections::HashSet;

    use chrono::Duration;
    use chrono::NaiveDate;

    use super::HyperEdge;
    use super::Nodes;
    use super::ScheduleGraph;
    use super::Skills;
    use crate::schedule_graph::EdgeType;
    use crate::schedule_graph::Period;
    use crate::schedule_graph::ScheduleGraphErrors;
    use crate::work_order::Activity;
    use crate::work_order::WorkOrder;

    #[test]
    fn make_a_hyper_edge()
    {
        let _hyper_edge = HyperEdge {
            edge_type: super::EdgeType::Assign,
            nodes: vec![1, 2, 3],
        };
    }

    #[test]
    fn test_schedule_graph_new()
    {
        let mut schedule_graph = ScheduleGraph::new();

        let date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        let index_worker = schedule_graph.add_node(Nodes::Worker(1234));
        let index_workorder = schedule_graph.add_node(Nodes::WorkOrder(1122334455));
        let index_period = schedule_graph.add_period(Period(date)).unwrap();

        assert!(schedule_graph.nodes[index_worker] == Nodes::Worker(1234));
        assert!(schedule_graph.nodes[index_workorder] == Nodes::WorkOrder(1122334455));
        assert!(schedule_graph.nodes[index_period] == Nodes::Period(Period(date)));

        schedule_graph.add_assignment(1234, 1122334455, Period(date)).unwrap();
    }

    #[test]
    fn test_add_work_order()
    {
        let mut schedule_graph = ScheduleGraph::new();

        schedule_graph.add_node(Nodes::Skill(Skills::MtnMech));

        let work_order = WorkOrder::new(
            1122334455,
            NaiveDate::from_ymd_opt(2025, 1, 13).unwrap(),
            vec![
                Activity::new(10, Skills::MtnMech),
                Activity::new(20, Skills::MtnMech),
                Activity::new(30, Skills::MtnMech),
            ],
        )
        .unwrap();

        let node_id = schedule_graph.add_work_order(&work_order).unwrap();

        assert_eq!(schedule_graph.nodes[node_id], Nodes::WorkOrder(1122334455));

        // let neighbors = schedule_graph..neighbors(node_id).collect::<Vec<_>>();

        assert_eq!(schedule_graph.nodes[node_id + 1], Nodes::Activity(10));
        assert_eq!(schedule_graph.nodes[node_id + 2], Nodes::Activity(20));
        assert_eq!(schedule_graph.nodes[node_id + 3], Nodes::Activity(30));

        let _edge_index = schedule_graph.adjacency[node_id + 1]
            .iter()
            .find(|e| {
                schedule_graph.hyperedges[**e]
                    == HyperEdge {
                        edge_type: EdgeType::FinishStart,
                        nodes: vec![node_id + 1, node_id + 2],
                    }
            })
            .unwrap();
        let _edge_index = schedule_graph.adjacency[node_id + 2]
            .iter()
            .find(|e| {
                schedule_graph.hyperedges[**e]
                    == HyperEdge {
                        edge_type: EdgeType::FinishStart,
                        nodes: vec![node_id + 2, node_id + 3],
                    }
            })
            .unwrap();
        assert!(!schedule_graph.adjacency[node_id + 3].iter().any(|e| {
            schedule_graph.hyperedges[*e]
                == HyperEdge {
                    edge_type: EdgeType::FinishStart,
                    nodes: vec![node_id + 3, node_id + 4],
                }
        }));
    }

    #[test]
    fn test_neighbors()
    {
        let mut schedule_graph = ScheduleGraph::new();

        let date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        let node = Nodes::Worker(1234);
        let index_worker_1 = schedule_graph.add_node(node.clone());
        let node1 = Nodes::WorkOrder(1122334455);
        let index_workorder_1 = schedule_graph.add_node(node1.clone());
        let node2 = Nodes::Period(Period(date));
        let index_period_1 = schedule_graph.add_node(node2.clone());

        assert!(schedule_graph.nodes[index_worker_1] == node);
        assert!(schedule_graph.nodes[index_workorder_1] == node1);
        assert!(schedule_graph.nodes[index_period_1] == node2);
        schedule_graph.add_assignment(1234, 1122334455, Period(date)).unwrap();

        let node3 = Nodes::Worker(1236);
        let index_worker_2 = schedule_graph.add_node(node3.clone());
        let node4 = Nodes::WorkOrder(1122334456);
        let index_workorder_2 = schedule_graph.add_node(node4.clone());

        assert!(schedule_graph.nodes[index_worker_2] == node3);
        assert!(schedule_graph.nodes[index_workorder_2] == node4);
        assert!(schedule_graph.nodes[index_period_1] == node2);
        schedule_graph.add_assignment(1236, 1122334456, Period(date)).unwrap();

        let assignment_edges = schedule_graph.find_all_assignments_for_period(Period(date)).unwrap();

        assert_eq!(
            assignment_edges[0],
            HyperEdge {
                edge_type: EdgeType::Assign,
                nodes: vec![0, 1, 2]
            }
        );

        assert_eq!(
            assignment_edges[1],
            HyperEdge {
                edge_type: EdgeType::Assign,
                nodes: vec![3, 4, 2]
            }
        );
    }

    #[test]
    fn test_skill_assign()
    {
        let mut schedule_graph = ScheduleGraph::new();

        let _worker_node = schedule_graph.add_node(Nodes::Worker(1234));
        let _skill_node = schedule_graph.add_node(Nodes::Skill(Skills::MtnMech));

        assert!(schedule_graph.add_assign_skill_to_worker(1234, super::Skills::MtnMech).is_ok());
        assert_eq!(
            schedule_graph.add_assign_skill_to_worker(1234, super::Skills::MtnElec),
            Err(ScheduleGraphErrors::SkillMissing)
        );
    }

    fn test_add_period()
    {
        let mut schedule_state = ScheduleGraph::new();

        let period_1 = Period(NaiveDate::from_ymd_opt(2025, 1, 13).unwrap());
        let period_2 = Period(NaiveDate::from_ymd_opt(2025, 1, 27).unwrap());
        let period_3 = Period(NaiveDate::from_ymd_opt(2025, 2, 10).unwrap());

        let _node_id = schedule_state.add_period(period_1).unwrap();
        let _node_id = schedule_state.add_period(period_2).unwrap();
        let _node_id = schedule_state.add_period(period_3).unwrap();

        let node_id = schedule_state.add_period(period_3);

        assert!(schedule_state.period_indices.contains_key(&period_1));
        assert!(schedule_state.period_indices.contains_key(&period_2));
        assert!(schedule_state.period_indices.contains_key(&period_3));

        assert!(node_id == Err(ScheduleGraphErrors::PeriodDuplicate));
        let start_date = NaiveDate::from_ymd_opt(2025, 1, 13).unwrap();
        let finish_date = NaiveDate::from_ymd_opt(2025, 2, 23).unwrap();

        let mut date = start_date;
        while date <= finish_date {
            assert!(schedule_state.day_indices.contains_key(&date), "Missing date: {date}");
            date += Duration::days(1);
        }

        let hash_set_days = schedule_state
            .nodes
            .iter()
            .filter(|&e| matches!(e, Nodes::Day(_)))
            .collect::<HashSet<_>>();

        let vec_days = schedule_state.nodes.iter().filter(|&e| matches!(e, Nodes::Day(_))).collect::<Vec<_>>();

        assert_eq!(hash_set_days.len(), vec_days.len())
    }
}
