use std::collections::HashMap;
use std::collections::HashSet;

use schedule_hypergraph::schedule_graph::Period;
use schedule_hypergraph::schedule_graph::TechnicianId;
use scheduling_environment::technician::Skill;
use scheduling_environment::work_order::Work;
use scheduling_environment::work_order::WorkOrderNumber;

#[derive(Debug)]
pub struct StrategicParameters
{
    pub strategic_work_order_parameters: HashMap<WorkOrderNumber, WorkOrderParameter>,
    pub strategic_capacity: StrategicResources,
    // pub strategic_clustering: StrategicClustering,
    pub period_locks: HashSet<Period>,

    // TODO #04 #00 #01
    // enum PeriodState {
    //     Previous(Period),
    //     Frozen(Period),
    //     Draft(Period),
    //     Draft2(Period),
    // }
    // Create this and have it change based on the value
    // of the [`SystemClock`].
    pub strategic_periods: Vec<Period>,
    // TODO [ ] Should the options be here? Yes they, no they should not.
    // WARN: Now you know why!
    // pub strategic_options: StrategicOptions,
}

// Okay, this is beginning to look like the right kind of thing
// now. It is crucial that you pace yourself and do not make the
// mistake of losing faith.
#[derive(Debug, PartialEq, Clone)]
pub struct WorkOrderParameter
{
    pub locked_in_period: Option<Period>,
    pub excluded_periods: HashSet<Period>,
    pub latest_period: Period,

    pub weight: i64,
    // This weight is derived from the ['StrategicOptions`]. This means that the code should
    // work better
    pub work_load: HashMap<Skill, Work>,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct StrategicResources(pub HashMap<Period, HashMap<TechnicianId, OperationalResource>>);

#[derive(Clone, PartialEq, Debug, Default)]
pub struct OperationalResource
{
    pub id: TechnicianId,
    pub total_hours: Work,
    pub skill_hours: HashMap<Skill, Work>,
}
