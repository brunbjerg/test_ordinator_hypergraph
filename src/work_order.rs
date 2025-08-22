use std::collections::HashSet;

use chrono::NaiveDate;
use chrono::TimeDelta;

use crate::schedule_graph::Skills;

pub type WorkOrderNumber = u64;

pub type ActivityNumber = u64;
#[derive(Hash, Clone, Copy, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub struct Activity
{
    activity_number: ActivityNumber,
    resource: Skills,
}

impl Activity
{
    pub fn number(&self) -> ActivityNumber
    {
        self.activity_number
    }

    pub fn skill(&self) -> Skills
    {
        self.resource
    }
}

impl Activity
{
    pub fn new(activity_number: u64, resource: Skills) -> Self
    {
        Self { activity_number, resource }
    }
}
pub struct WorkOrder
{
    number: WorkOrderNumber,
    basic_start_date: NaiveDate,
    activities: Vec<Activity>,
}

#[derive(Debug)]
pub enum WorkOrderError
{
    InvalidWorkOrderNumber(String),
    NonSortedActivities(Vec<Activity>),
    DuplicatedActivities,
}

impl WorkOrder
{
    pub fn new(number: WorkOrderNumber, basic_start_date: NaiveDate, activities: Vec<Activity>) -> Result<Self, WorkOrderError>
    {
        if number.to_string().len() != 10 {
            return Err(WorkOrderError::InvalidWorkOrderNumber(number.to_string()));
        }

        if !activities.is_sorted() {
            return Err(WorkOrderError::NonSortedActivities(activities));
        }

        if activities.iter().collect::<HashSet<_>>().len() != activities.len() {
            return Err(WorkOrderError::DuplicatedActivities);
        }

        Ok(Self {
            number,
            activities,
            basic_start_date,
        })
    }

    pub fn number(&self) -> WorkOrderNumber
    {
        self.number
    }

    pub fn activities(&self) -> &Vec<Activity>
    {
        &self.activities
    }

    pub(crate) fn activities_relations(&self) -> Vec<ActivityRelation>
    {
        (0..self.activities.len()).map(|_| ActivityRelation::FinishStart).collect()
    }
}
pub enum ActivityRelation
{
    StartStart,
    FinishStart,
    Postpone(TimeDelta),
}
