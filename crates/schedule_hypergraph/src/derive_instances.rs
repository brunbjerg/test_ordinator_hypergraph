use scheduling_environment::work_order::WorkOrderNumber;

use crate::schedule_graph::HyperEdge;
use crate::schedule_graph::Node;
use crate::schedule_graph::ScheduleGraph;

/// This contains the API for deriving problem instances for the
/// optimization algorithms.
///
/// Every methods has to be non-mutating
impl ScheduleGraph
{
    pub fn derive_work_orders(&self, work_order_numbers: &[WorkOrderNumber]) -> GraphWorkOrders
    {
        for node in self.nodes() {
            // ESSAY:
            // What is the information that you want to get out here?
            //
            // You want to extract all the
            // QUESTION: How to get the hours?
            // ANSWER: For each Technician use the availabilities to
            // calculate the correct
            match node {
                Node::Technician(e) => todo!(),
                Node::WorkOrder(work_order_node_index) => {
                    // Derive every hyperedge
                    let hyperedge_indices = &self.incidence_list()[*work_order_node_index as usize];

                    // Now we have to find each of the assignments
                    //

                    for hyperedge_index in hyperedge_indices {
                        let hyperedge = self.hyperedges()[*hyperedge_index];

                        match hyperedge {}
                    }
                }
                Node::Activity(activity_node) => todo!(),
                Node::Period(period) => todo!(),
                Node::Skill(skill) => todo!(),
                Node::Day(naive_date) => todo!(),
            }
        }

        GraphWorkOrders {}
    }
}

struct GraphWorkOrders {}
