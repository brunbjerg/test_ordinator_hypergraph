use std::collections::HashSet;

use chrono::NaiveDate;
use chrono::TimeDelta;

use crate::technician::Skill;

pub type WorkOrderNumber = u64;
pub type NumberOfPeople = u64;

pub type ActivityNumber = u64;
pub type Work = f64;
#[derive(Hash, Clone, Copy, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub struct Activity
{
    activity_number: ActivityNumber,
    number_of_people: NumberOfPeople,
    resource: Skill,
}

impl Activity
{
    pub fn activity_number(&self) -> ActivityNumber
    {
        self.activity_number
    }

    pub fn skill(&self) -> Skill
    {
        self.resource
    }

    pub fn number_of_people(&self) -> NumberOfPeople
    {
        self.number_of_people
    }
}

impl Activity
{
    pub fn new(activity_number: u64, number_of_people: NumberOfPeople, resource: Skill) -> Self
    {
        Self {
            activity_number,
            resource,
            number_of_people,
        }
    }
}
pub struct WorkOrder
{
    work_order_number: WorkOrderNumber,
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
    pub fn new(work_order_number: WorkOrderNumber, basic_start_date: NaiveDate, activities: Vec<Activity>) -> Result<Self, WorkOrderError>
    {
        if work_order_number.to_string().len() != 10 {
            return Err(WorkOrderError::InvalidWorkOrderNumber(work_order_number.to_string()));
        }

        if !activities.is_sorted() {
            return Err(WorkOrderError::NonSortedActivities(activities));
        }

        if activities.iter().collect::<HashSet<_>>().len() != activities.len() {
            return Err(WorkOrderError::DuplicatedActivities);
        }

        Ok(Self {
            work_order_number,
            activities,
            basic_start_date,
        })
    }

    pub fn work_order_number(&self) -> WorkOrderNumber
    {
        self.work_order_number
    }

    pub fn activities(&self) -> &Vec<Activity>
    {
        &self.activities
    }

    pub fn activities_relations(&self) -> Vec<ActivityRelation>
    {
        (0..self.activities.len()).map(|_| ActivityRelation::FinishStart).collect()
    }

    pub fn basic_start(&self) -> NaiveDate
    {
        self.basic_start_date
    }
}
pub enum ActivityRelation
{
    StartStart,
    FinishStart,
    Postpone(TimeDelta),
}
