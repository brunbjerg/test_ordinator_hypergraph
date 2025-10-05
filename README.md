# Ordinator Hypergraph Aggregate


This is a crate for Ordinator that will function as a domain specific hypergraph
for the storing and validation complex interactions in  data for Ordinator.

The coordinator of the `ScheduleGraph` will be handled by sagas and orchestrators
in the code.

### TODOs 

Handle 
- [x] Implement the `basic_start_date`

Take a break now!
- [ ] Implement the bidirectional `incidence` list
- [ ] Implement `Technician`/`Worker` aggregate
- [ ] Apply constant size vecs




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
