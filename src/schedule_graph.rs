use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::hash_map::Entry;

use chrono::Duration;
use chrono::NaiveDate;
use chrono::NaiveTime;
use tracing::debug;

use crate::technician::Technician;
use crate::work_order::ActivityNumber;
use crate::work_order::ActivityRelation;
use crate::work_order::WorkOrder;
use crate::work_order::WorkOrderNumber;

// Type Alias to make reasoning about the indices easier
pub type NodeIndex = usize;
pub type EdgeIndex = usize;
pub type TechnicianId = usize;
pub type StartTime = NaiveTime;
pub type FinishTime = NaiveTime;

#[derive(Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub enum ScheduleGraphErrors
{
    ActivityMissing,
    DayMissing,
    PeriodDuplicate,
    PeriodMissing,
    SkillMissing,
    WorkOrderActivityMissingSkills,
    WorkOrderDuplicate,
    WorkOrderMissing,
    WorkerMissing,
    WorkerDuplicate,
}

#[derive(Hash, Copy, Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub struct Period(NaiveDate);

#[derive(Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub struct HyperEdge
{
    edge_type: EdgeType,
    nodes: Vec<NodeIndex>,
}

#[derive(Hash, Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub enum Node
{
    Technician(TechnicianId),
    WorkOrder(WorkOrderNumber),
    Activity(ActivityNumber),
    Period(Period),
    Skill(Skill),
    Day(NaiveDate),
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub enum Skill
{
    MtnMech,
    MtnElec,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub enum EdgeType
{
    /// Date specific
    Assign(Option<(StartTime, FinishTime)>),
    Available,
    Exclude,
    BasicStart,

    Contains,
    Requires,
    StartStart,
    FinishStart,
    /// Has skill
    HasSkill,
}

#[derive(Debug)]
pub struct ScheduleGraph
{
    /// Nodes of the problem
    nodes: Vec<Node>,

    /// Hyperedges to handle all the complex interactions
    hyperedges: Vec<HyperEdge>,

    /// Adjacency list
    incidence_list: Vec<Vec<EdgeIndex>>,

    /// Indices to look up nodes
    worker_indices: HashMap<TechnicianId, NodeIndex>,
    work_order_indices: HashMap<WorkOrderNumber, NodeIndex>,
    period_indices: HashMap<Period, NodeIndex>,
    skill_indices: HashMap<Skill, NodeIndex>,
    day_indices: BTreeMap<NaiveDate, NodeIndex>,
}

/// Public methods
impl ScheduleGraph
{
    pub fn new() -> Self
    {
        Self {
            nodes: vec![],
            hyperedges: vec![],
            incidence_list: vec![],
            worker_indices: HashMap::new(),
            work_order_indices: HashMap::new(),
            period_indices: HashMap::new(),
            skill_indices: HashMap::new(),
            day_indices: BTreeMap::new(),
        }
    }
}

// impl ScheduleGraph {
//     pub fn work_order_relations(&self, work_order: &WorkOrder) ->
// Result<Vec<()>> }

/// Public API to add [`Nodes`] to the graph.
impl ScheduleGraph
{
    pub fn add_work_order(&mut self, work_order: &WorkOrder) -> Result<NodeIndex, ScheduleGraphErrors>
    {
        if !work_order
            .activities()
            .iter()
            .all(|activity| self.skill_indices.keys().any(|&all_skills| all_skills == activity.skill()))
        {
            return Err(ScheduleGraphErrors::WorkOrderActivityMissingSkills);
        }

        let day_node = *self.day_indices.get(&work_order.basic_start()).ok_or(ScheduleGraphErrors::DayMissing)?;

        // Crucial lesson! This cannot come first! You learned something great here!
        let work_order_node = match self.work_order_indices.entry(work_order.work_order_number()) {
            Entry::Vacant(_new_work_order) => self.add_node(Node::WorkOrder(work_order.work_order_number())),
            Entry::Occupied(_already_inserted_work_order) => return Err(ScheduleGraphErrors::WorkOrderDuplicate),
        };

        let _basic_start_edge = self.add_edge(EdgeType::BasicStart, vec![work_order_node, day_node]);

        let mut previous_activity_node = usize::MAX;
        let activity_relations = work_order.activities_relations();
        for (activity_index, activity) in work_order.activities().iter().enumerate() {
            let activity_node = self.add_node(Node::Activity(activity.activity_number()));
            dbg!(activity, activity_node);
            let skill_node = *self.skill_indices.get(&activity.skill()).ok_or(ScheduleGraphErrors::SkillMissing)?;

            dbg!(skill_node);
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

        // TODO [x] - add relationships between activities here.

        self.work_order_indices.insert(work_order.work_order_number(), work_order_node);
        Ok(work_order_node)
    }

    pub fn add_period(&mut self, period: Period) -> Result<NodeIndex, ScheduleGraphErrors>
    {
        if self.period_indices.contains_key(&period) {
            return Err(ScheduleGraphErrors::PeriodDuplicate);
        };

        let days_in_period = (0..14).map(|e| period.0 + chrono::Days::new(e)).collect::<Vec<_>>();

        for day in days_in_period {
            let day_node = self.add_node(Node::Day(day));
            self.day_indices.insert(day, day_node);
        }

        let node_id = self.add_node(Node::Period(period));

        self.period_indices.insert(period, node_id);
        Ok(node_id)
    }

    // TODO [ ] - Start here when ready again.
    pub fn add_technician(&mut self, technician: Technician) -> Result<NodeIndex, ScheduleGraphErrors>
    {
        if self.worker_indices.contains_key(&technician.id()) {
            return Err(ScheduleGraphErrors::WorkerDuplicate);
        }

        let mut skills = vec![];
        for skill in technician.skills() {
            let skill = self.skill_indices.get(skill).ok_or(ScheduleGraphErrors::SkillMissing)?;

        }

        let availabilities: Vec<Vec<NaiveDate>> = vec![];
        for start_and_finish_dates in technician.availabilities() {
            let single_availability = vec![];
            for date in start_and_finish_dates.
            let start_date = self.day_indices.get(&start_and_finish_dates.0.date()).ok_or(ScheduleGraphErrors::DayMissing)?;
            let finish_date = self.day_indices.get(&start_and_finish_dates.1.date()).ok_or(ScheduleGraphErrors::DayMissing)?;

        }


        let technician_id = self.add_node(Node::Technician(technician.id()));

        let skill_edge = self.add_edge(EdgeType::HasSkill, )

        
    }
}

/// Public API to add [`HyperEdges`] to the graph
impl ScheduleGraph
{
    // TODO [ ] - this should be formulated as ids... it should be the types that
    // are found inside of the `Nodes` enum variants.
    pub fn add_assignment_work_order(
        &mut self,
        worker: TechnicianId,
        work_order: WorkOrderNumber,
        date: Period,
    ) -> Result<EdgeIndex, ScheduleGraphErrors>
    {
        // This should return an error if the `Nodes` is not present.
        let worker = self.worker_indices.get(&worker).ok_or(ScheduleGraphErrors::WorkerMissing)?;
        let work_order = self.work_order_indices.get(&work_order).ok_or(ScheduleGraphErrors::WorkOrderMissing)?;
        let date = self.period_indices.get(&date).ok_or(ScheduleGraphErrors::PeriodMissing)?;

        let hyperedge = HyperEdge {
            edge_type: EdgeType::Assign(None),
            nodes: vec![*worker, *work_order, *date],
        };

        self.hyperedges.push(hyperedge);
        Ok(self.hyperedges.len() - 1)
    }

    pub fn add_assignment_activity(
        &mut self,
        worker: TechnicianId,
        work_order_number: WorkOrderNumber,
        activity_number: ActivityNumber,
        days: Vec<NaiveDate>,
        start_and_finish_time: (StartTime, FinishTime),
    ) -> Result<EdgeIndex, ScheduleGraphErrors>
    {
        let worker_node_id = self.worker_indices.get(&worker).ok_or(ScheduleGraphErrors::WorkerMissing)?;
        let work_order_node_id = self
            .work_order_indices
            .get(&work_order_number)
            .ok_or(ScheduleGraphErrors::WorkOrderMissing)?;

        // TODO - [ ] Make a `nodes_in_hyperedge(self, edge_id) -> Vec<Nodes>` method.
        let activity_node_id = self
            .incidence_list
            .get(*work_order_node_id)
            .ok_or(ScheduleGraphErrors::WorkOrderMissing)?
            .iter()
            .find_map(|&edge_id| {
                self.hyperedges[edge_id]
                    .nodes
                    .iter()
                    .position(|&e| self.nodes[e] == Node::Activity(activity_number))
            })
            .ok_or(ScheduleGraphErrors::ActivityMissing)?;

        let mut date_node_ids = vec![];
        for naive_date in days {
            date_node_ids.push(self.day_indices.get(&naive_date).ok_or(ScheduleGraphErrors::DayMissing)?);
        }

        Ok(self.add_edge(EdgeType::Assign(Some(start_and_finish_time)), vec![*worker_node_id, activity_node_id]))
    }

    // This function should be in a different place in the code. I believe that
    // this is an internal helper function. The user should not be exposed to a
    // `HyperEdge` instance. It should return `Vec<Workers>` or `Vec<WorkOrder>`
    // or `Vec<WorkOrderActivities>`. This should be moved to an Internal API
    // function call.

    /// If the start_naive_date of `EdgeType::Assign(assignment)` in the period
    /// interval the it counts as belonging to that period.
    pub fn find_all_assignments_for_period(&self, period_start_date: Period) -> Result<Vec<EdgeIndex>, ScheduleGraphErrors>
    {
        if !self.nodes.iter().any(|e| e == &Node::Period(period_start_date)) {
            return Err(ScheduleGraphErrors::PeriodMissing);
        }
        let assignment_hyper_edges = self
            .hyperedges
            .iter()
            .enumerate()
            .filter(|e| matches!(e.1.edge_type, EdgeType::Assign(_)))
            .collect::<Vec<_>>();

        let mut edges = vec![];
        for (edge_index, hyper_edge) in &assignment_hyper_edges {
            for nodes in &hyper_edge.nodes {
                match self.nodes[*nodes] {
                    Node::Period(period) => {
                        if period == period_start_date {
                            edges.push(*edge_index)
                        }
                    }
                    Node::Day(naive_date) => {
                        if period_start_date.0 <= naive_date && naive_date < (period_start_date.0 + Duration::days(13)) {
                            edges.push(*edge_index)
                        }
                    }
                    // We are only interested in the time of the assignment. `Worker` and `WorkOrder` belong
                    // in a different method.
                    _ => (),
                }
            }
        }

        Ok(edges)
    }

    pub fn add_assign_skill_to_worker(&mut self, worker: TechnicianId, skill: Skill) -> Result<EdgeIndex, ScheduleGraphErrors>
    {
        let worker = self.worker_indices.get(&worker).ok_or(ScheduleGraphErrors::WorkerMissing)?;
        let skill = self.skill_indices.get(&skill).ok_or(ScheduleGraphErrors::SkillMissing)?;

        Ok(self.add_edge(EdgeType::HasSkill, vec![*worker, *skill]))
    }

    /// This method can fail when:
    /// * `WorkOrderNumber` does not exist
    /// * `Period` does not exist.
    /// * The hyperedge between the `WorkOrderNumber` and `Period` already
    ///   exists.
    pub fn add_exclusion(&mut self, work_order_number: &WorkOrderNumber, period: &Period) -> Result<EdgeIndex, ScheduleGraphErrors>
    {
        let work_order_node_id = self
            .work_order_indices
            .get(work_order_number)
            .ok_or(ScheduleGraphErrors::WorkOrderMissing)?;
        let period_node_id = self.period_indices.get(period).ok_or(ScheduleGraphErrors::PeriodMissing)?;

        Ok(self.add_edge(EdgeType::Exclude, vec![*work_order_node_id, *period_node_id]))
    }
}

/// Private methods.
///
/// [`NodeIndex`] and [`EdgeIndex`] are not allowed to be a part of the
/// public API of the type. The graph should only expose domain types
/// found in `ordinator-scheduling-environment`
impl ScheduleGraph
{
    fn add_node(&mut self, node: Node) -> NodeIndex
    {
        // This is the next element as `len()` is one larger than the last index
        let node_index = self.nodes.len();
        let none_checker = match node {
            Node::Technician(worker) => self.worker_indices.insert(worker, node_index),
            Node::WorkOrder(work_order) => self.work_order_indices.insert(work_order, node_index),
            Node::Period(naive_date) => self.period_indices.insert(naive_date, node_index),
            Node::Skill(skills) => self.skill_indices.insert(skills, node_index),
            Node::Activity(a) => {
                debug!(target: "developer", activity = a, "No node index for `Activities`");
                None
            }
            Node::Day(naive_date) => self.day_indices.insert(naive_date, node_index),
        };
        assert!(none_checker.is_none());

        self.incidence_list.push(vec![]);

        // node is added `Vec<Nodes>`
        self.nodes.push(node);
        node_index
    }

    fn add_edge(&mut self, edge_type: EdgeType, nodes: Vec<NodeIndex>) -> EdgeIndex
    {
        let edge_index = self.hyperedges.len();

        for node_index in &nodes {
            self.incidence_list[*node_index].push(edge_index);
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
    use super::Node;
    use super::ScheduleGraph;
    use super::Skill;
    use crate::schedule_graph::EdgeType;
    use crate::schedule_graph::Period;
    use crate::schedule_graph::ScheduleGraphErrors;
    use crate::work_order::Activity;
    use crate::work_order::WorkOrder;

    #[test]
    fn test_schedule_graph_new()
    {
        let mut schedule_graph = ScheduleGraph::new();

        let date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        let index_worker = schedule_graph.add_node(Node::Technician(1234));
        let index_workorder = schedule_graph.add_node(Node::WorkOrder(1122334455));
        let index_period = schedule_graph.add_period(Period(date)).unwrap();

        assert!(schedule_graph.nodes[index_worker] == Node::Technician(1234));
        assert!(schedule_graph.nodes[index_workorder] == Node::WorkOrder(1122334455));
        assert!(schedule_graph.nodes[index_period] == Node::Period(Period(date)));

        schedule_graph.add_assignment_work_order(1234, 1122334455, Period(date)).unwrap();
    }

    #[test]
    fn test_add_work_order()
    {
        let mut schedule_graph = ScheduleGraph::new();

        let _skill_node_id = schedule_graph.add_node(Node::Skill(Skill::MtnMech));

        let basic_start_date = NaiveDate::from_ymd_opt(2025, 1, 13).unwrap();
        let work_order = WorkOrder::new(
            1122334455,
            basic_start_date,
            vec![
                Activity::new(10, 1, Skill::MtnMech),
                Activity::new(20, 1, Skill::MtnMech),
                Activity::new(30, 1, Skill::MtnMech),
            ],
        )
        .unwrap();

        assert_eq!(schedule_graph.add_work_order(&work_order), Err(ScheduleGraphErrors::DayMissing));

        let _period_node_id = schedule_graph.add_period(Period(basic_start_date)).unwrap();
        let work_order_node_id = schedule_graph.add_work_order(&work_order).expect("Could not add work order");

        assert_eq!(schedule_graph.nodes[work_order_node_id], Node::WorkOrder(1122334455));

        // let neighbors = schedule_graph..neighbors(node_id).collect::<Vec<_>>();

        assert_eq!(schedule_graph.nodes[work_order_node_id + 1], Node::Activity(10));
        assert_eq!(schedule_graph.nodes[work_order_node_id + 2], Node::Activity(20));
        assert_eq!(schedule_graph.nodes[work_order_node_id + 3], Node::Activity(30));

        let _edge_index = schedule_graph.incidence_list[work_order_node_id + 1]
            .iter()
            .find(|e| {
                schedule_graph.hyperedges[**e]
                    == HyperEdge {
                        edge_type: EdgeType::FinishStart,
                        nodes: vec![work_order_node_id + 1, work_order_node_id + 2],
                    }
            })
            .unwrap();
        let _edge_index = schedule_graph.incidence_list[work_order_node_id + 2]
            .iter()
            .find(|e| {
                schedule_graph.hyperedges[**e]
                    == HyperEdge {
                        edge_type: EdgeType::FinishStart,
                        nodes: vec![work_order_node_id + 2, work_order_node_id + 3],
                    }
            })
            .unwrap();
        assert!(!schedule_graph.incidence_list[work_order_node_id + 3].iter().any(|e| {
            schedule_graph.hyperedges[*e]
                == HyperEdge {
                    edge_type: EdgeType::FinishStart,
                    nodes: vec![work_order_node_id + 3, work_order_node_id + 4],
                }
        }));

        let basic_start_day_node_id = *schedule_graph.day_indices.get(&basic_start_date).unwrap();

        dbg!(
            &schedule_graph.incidence_list,
            basic_start_day_node_id,
            work_order_node_id,
            &schedule_graph.incidence_list[work_order_node_id],
            &schedule_graph.day_indices,
        );

        let work_order_edge_ids = &schedule_graph.incidence_list[work_order_node_id];

        for edge_id in work_order_edge_ids {
            let hyper_edge = &schedule_graph.hyperedges[*edge_id];
            let edge_type = &hyper_edge.edge_type;
            let nodes = &hyper_edge.nodes;
            match edge_type {
                EdgeType::Assign(_) => todo!(),
                EdgeType::Available => todo!(),
                EdgeType::BasicStart => {
                    assert_eq!(basic_start_day_node_id, nodes[1]);
                    assert_eq!(work_order_node_id, nodes[0]);
                }
                EdgeType::Contains => {
                    assert_eq!(work_order_node_id, nodes[0]);
                }
                EdgeType::Requires => todo!(),
                EdgeType::StartStart => todo!(),
                EdgeType::FinishStart => todo!(),
                EdgeType::Exclude => todo!(),
                EdgeType::HasSkill => todo!(),
            }
        }

        // assert!(day_node == *basic_start_day_node);
        // assert_eq!(period_node_incidence, period_node_id);
    }

    #[test]
    fn test_neighbors()
    {
        let mut schedule_graph = ScheduleGraph::new();

        let date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        let node = Node::Technician(1234);
        let index_worker_1 = schedule_graph.add_node(node.clone());
        let node1 = Node::WorkOrder(1122334455);
        let index_workorder_1 = schedule_graph.add_node(node1.clone());
        let node2 = Node::Period(Period(date));
        let index_period_1 = schedule_graph.add_node(node2.clone());

        assert!(schedule_graph.nodes[index_worker_1] == node);
        assert!(schedule_graph.nodes[index_workorder_1] == node1);
        assert!(schedule_graph.nodes[index_period_1] == node2);
        let assignment_edge_index_0 = schedule_graph.add_assignment_work_order(1234, 1122334455, Period(date)).unwrap();

        let node3 = Node::Technician(1236);
        let index_worker_2 = schedule_graph.add_node(node3.clone());
        let node4 = Node::WorkOrder(1122334456);
        let index_workorder_2 = schedule_graph.add_node(node4.clone());

        assert!(schedule_graph.nodes[index_worker_2] == node3);
        assert!(schedule_graph.nodes[index_workorder_2] == node4);
        assert!(schedule_graph.nodes[index_period_1] == node2);
        let assignment_edge_index_1 = schedule_graph.add_assignment_work_order(1236, 1122334456, Period(date)).unwrap();

        let assignment_edges = schedule_graph.find_all_assignments_for_period(Period(date)).unwrap();

        assert_eq!(assignment_edges[0], assignment_edge_index_0);

        assert_eq!(assignment_edges[1], assignment_edge_index_1);
    }

    #[test]
    fn test_skill_assign()
    {
        let mut schedule_graph = ScheduleGraph::new();

        let _worker_node = schedule_graph.add_node(Node::Technician(1234));
        let _skill_node = schedule_graph.add_node(Node::Skill(Skill::MtnMech));

        assert!(schedule_graph.add_assign_skill_to_worker(1234, super::Skill::MtnMech).is_ok());
        assert_eq!(
            schedule_graph.add_assign_skill_to_worker(1234, super::Skill::MtnElec),
            Err(ScheduleGraphErrors::SkillMissing)
        );
    }

    #[test]
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

        let hash_set_days = schedule_state.nodes.iter().filter(|&e| matches!(e, Node::Day(_))).collect::<HashSet<_>>();

        let vec_days = schedule_state.nodes.iter().filter(|&e| matches!(e, Node::Day(_))).collect::<Vec<_>>();

        assert_eq!(hash_set_days.len(), vec_days.len())
    }

    #[test]
    fn test_multi_directional_hypergraph()
    {
        let mut schedule_graph = ScheduleGraph::new();

        let node_0 = Node::WorkOrder(1111990000);
        let node_1 = Node::WorkOrder(1111990001);
        let node_2 = Node::WorkOrder(1111990002);
        let node_3 = Node::WorkOrder(1111990003);
        let node_4 = Node::WorkOrder(1111990004);
        let node_5 = Node::WorkOrder(1111990005);
        let node_6 = Node::WorkOrder(1111990006);
        let node_7 = Node::WorkOrder(1111990007);

        let node_index_0 = schedule_graph.add_node(node_0);
        let node_index_1 = schedule_graph.add_node(node_1);
        let node_index_2 = schedule_graph.add_node(node_2);
        let node_index_3 = schedule_graph.add_node(node_3);
        let node_index_4 = schedule_graph.add_node(node_4);
        let node_index_5 = schedule_graph.add_node(node_5);
        let node_index_6 = schedule_graph.add_node(node_6);
        let node_index_7 = schedule_graph.add_node(node_7);

        let edge_index_0 = schedule_graph.add_edge(EdgeType::Assign(None), vec![0, 2, 4, 6]);
        let edge_index_1 = schedule_graph.add_edge(EdgeType::Assign(None), vec![1, 3, 5, 7]);
        let edge_index_2 = schedule_graph.add_edge(EdgeType::Assign(None), vec![0, 3, 6]);

        assert_eq!(schedule_graph.incidence_list[node_index_0], vec![edge_index_0, edge_index_2]);
        assert_eq!(schedule_graph.incidence_list[node_index_1], vec![edge_index_1]);
        assert_eq!(schedule_graph.incidence_list[node_index_2], vec![edge_index_0]);
        assert_eq!(schedule_graph.incidence_list[node_index_3], vec![edge_index_1, edge_index_2]);
        assert_eq!(schedule_graph.incidence_list[node_index_4], vec![edge_index_0]);
        assert_eq!(schedule_graph.incidence_list[node_index_5], vec![edge_index_1]);
        assert_eq!(schedule_graph.incidence_list[node_index_6], vec![edge_index_0, edge_index_2]);
        assert_eq!(schedule_graph.incidence_list[node_index_7], vec![edge_index_1]);
    }

    #[test]
    fn test_add_exclusion()
    {
        let mut schedule_graph = ScheduleGraph::new();

        let basic_start_date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        let work_order = WorkOrder::new(1111990000, basic_start_date, vec![]).unwrap();

        let period = Period(basic_start_date);

        let period_node_id = schedule_graph.add_period(period).unwrap();
        let work_order_node_id = schedule_graph.add_work_order(&work_order).unwrap();

        let exclusion_edge = schedule_graph.add_exclusion(&1111990000, &period).unwrap();

        assert_eq!(
            schedule_graph.hyperedges[1],
            HyperEdge {
                edge_type: EdgeType::Exclude,
                nodes: vec![work_order_node_id, period_node_id]
            }
        );

        dbg!(schedule_graph.hyperedges.get(schedule_graph.incidence_list[work_order_node_id][0]));
        dbg!(schedule_graph.hyperedges.get(schedule_graph.incidence_list[work_order_node_id][1]));

        assert!(schedule_graph.incidence_list[work_order_node_id].contains(&exclusion_edge));
        assert!(schedule_graph.incidence_list[period_node_id].contains(&exclusion_edge));
    }
}
