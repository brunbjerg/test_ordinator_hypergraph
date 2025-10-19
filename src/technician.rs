use std::collections::BTreeSet;

use chrono::NaiveDate;
use chrono::NaiveDateTime;

use crate::schedule_graph::Skill;

#[derive(Debug, PartialEq)]
pub enum TechnicianBuilderError
{
    OverlappingAvailability
    {
        new_start: NaiveDateTime,
        new_end: NaiveDateTime,
        existing_start: NaiveDateTime,
        existing_end: NaiveDateTime,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Availability(NaiveDateTime, NaiveDateTime);

impl Availability
{
    pub fn new(start: NaiveDateTime, end: NaiveDateTime) -> Self
    {
        Self(start, end)
    }

    pub fn start(&self) -> NaiveDateTime
    {
        self.0
    }

    pub fn end(&self) -> NaiveDateTime
    {
        self.1
    }

    pub fn start_date(&self) -> NaiveDate
    {
        self.0.date()
    }

    pub fn finish_date(&self) -> NaiveDate
    {
        self.1.date()
    }

    /// Check if this availability overlaps with another
    pub fn overlaps_with(&self, other: &Availability) -> bool
    {
        // Two intervals overlap if: start1 < end2 && start2 < end1
        self.0 < other.1 && other.0 < self.1
    }
}

// Implement ordering traits for BTreeSet
impl PartialOrd for Availability
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering>
    {
        Some(self.cmp(other))
    }
}

impl Ord for Availability
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering
    {
        // Primary sort by start time, then by end time
        self.0.cmp(&other.0).then(self.1.cmp(&other.1))
    }
}

pub struct Technician
{
    technician_id: usize,
    availabilities: BTreeSet<Availability>,
    skills: BTreeSet<Skill>,
}

pub struct TechnicianBuilder
{
    technician_id: usize,
    availabilities: BTreeSet<Availability>,
    skills: BTreeSet<Skill>,
}

impl TechnicianBuilder
{
    pub fn new(technician_id: usize) -> Self
    {
        Self {
            technician_id,
            availabilities: BTreeSet::new(),
            skills: BTreeSet::new(),
        }
    }

    pub fn add_availability(mut self, start: NaiveDateTime, end: NaiveDateTime) -> Result<Self, TechnicianBuilderError>
    {
        let new_availability = Availability::new(start, end);

        // Check for overlaps with existing availabilities
        for existing in &self.availabilities {
            if new_availability.overlaps_with(existing) {
                return Err(TechnicianBuilderError::OverlappingAvailability {
                    new_start: start,
                    new_end: end,
                    existing_start: existing.start(),
                    existing_end: existing.end(),
                });
            }
        }

        self.availabilities.insert(new_availability);
        Ok(self)
    }

    pub fn add_skill(mut self, skill: Skill) -> Self
    {
        self.skills.insert(skill);
        self
    }

    pub fn build(self) -> Technician
    {
        Technician {
            technician_id: self.technician_id,
            availabilities: self.availabilities,
            skills: self.skills,
        }
    }
}

impl Technician
{
    pub fn builder(technician_id: usize) -> TechnicianBuilder
    {
        TechnicianBuilder::new(technician_id)
    }

    pub fn id(&self) -> usize
    {
        self.technician_id
    }

    pub fn skills(&self) -> Vec<&Skill>
    {
        self.skills.iter().collect()
    }

    pub fn availabilities(&self) -> Vec<&Availability>
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
