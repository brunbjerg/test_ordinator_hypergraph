# Ordinator Hypergraph Aggregate


This is a crate for Ordinator that will function as a domain specific hypergraph
for the storing and validation complex interactions in  data for Ordinator.

The coordinator of the `ScheduleGraph` will be handled by sagas and orchestrators
in the code.

### TODOs 

Handle 
- [x] Implement the `basic_start_date`
- [x] Implement the bidirectional `incidence` list
- [x] Implement `Technician`/`Worker` aggregate

Take a break now!
- [x] Test `add_activity_assignment`
- [x] Ensure every `technician` `availability` overlap with `assignment`

- [ ] Ensure activity `number_of_people` bound
- [ ] Add node separator in hyperedge
- [ ] Ensure that the `skill` of the `activity` matches the `technician`

- [ ] Add exclusion checker to the system
- [ ] Apply constant size vecs
- [ ] Make `ScheduleGraphBuilder`
- [ ] Make `Parameters` `impl` block




- [ ] Why are `roles` used?
```rust

pub struct HyperEdge { 
    // Most edges have 3-5 nodes, avoid heap allocation
    nodes: SmallVec<[u64; 4]>,  // Stack-allocated for ≤4 nodes
    roles: HashMap<Role, u64>,
}
```


### Performance
- [ ] Test `std::sync::RwLock` and `parking_lot::RwLock`
- [ ] Test `SmallVec`, `Vec<u64>` vs `Vec<EnumType>`


### Goal of the Library
This crate will implement a hypergraph to work with the
ordinator scheduling application



# Nodes
## WorkOrder
- [ ] `WorkOrders`
- [ ] `Activity`

## Technician
- [ ] `Technician`

## Skill
- [ ] `Skills`


## Time
- [ ] `Period`
- [ ] `Day`
- [ ] 

## Edges
- [ ] Assignment => { $`Activity`, @`Technician`, $`Period`, @`Days`}

- [ ] PeriodDay => {$`Period`, @`Day`,}

- [ ] WorkOrderActivity => {$`WorkOrder`, @`Activity`}

- [ ] ActivityRelation => {$`Activity`, $`Activity`}

- [ ] Exclude => {$`WorkOrder`, @`Period`}
- [ ] Exclude -> {$`Activity`, @`Day`}

### Interfaces
- [ ] `StrategicParameters`
- [ ] `TacticalParameters`
- [ ] `SupervisorParameters`
- [ ] `OperationalParameters`


What do you want to build now? I think that making functions to retrieve the
`Parameters` is a good idea! Yes I think that is the next goal here! Extracting
all the state from the `SchedulingEnvironment` is not a good idea. The best
thing to do is build the graph from the aggregates in the repos and then
have the algorithms only work on the Graph and then make sages 

# DESIGN DECISION: DEMARCATING NODES
This is a crucial point in the code, for every hyperedge you have to understand what the
best approach is for making the code work correctly when there is a
variable number of nodes of a specific type in the hyperedge.

1. Use a `usize::MAX` as a seperator
2. Put a struct into each `HyperEdge` enum variant.
3. Edge metadata

# DESIGN DECISION: UPDATING OF HYPEREDGES
I strongly feel that it should be illigal to mutate hyperedges. You would
have to make an enormous amount of conditionals to check the validity of the
object---meaning hyperedge.

Because you would have to check every possible partial configuration of the
objective that could occur from a mutating operation. This is not a good idea...

At the very least you would need to make it extremely clear which methods that
are mutating and which are not. You would have to make a standard interface for
handling these kind of things.

# 
