use std::collections::BTreeSet;

use chrono::NaiveDate;
use chrono::NaiveDateTime;

use crate::schedule_graph::Skill;

pub struct Technician
{
    technician_id: usize,
    availabilities: BTreeSet<(NaiveDateTime, NaiveDateTime)>,
    skills: BTreeSet<Skill>,
}

impl Technician
{
    pub fn id(&self) -> usize
    {
        self.technician_id
    }

    pub fn skills(&self) -> Vec<&Skill>
    {
        self.skills.iter().collect()
    }

    pub fn availabilities(&self) -> Vec<&(NaiveDateTime, NaiveDateTime)>
    {
        self.availabilities.iter().collect()
    }
}

// #[derive(Serialize, Deserialize)]
// pub struct Worker
// {
//     name: String,
//     id_worker: i32,
//     capacity: f64,
//     trait_: String,
//     availabilities: Vec<Availability>,
//     These will be handled by the relationships in the Graph.
//     assigned_activities: Vec<AssignedWork>,
// }
