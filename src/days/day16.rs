use crate::{
    parsing::{integer, line_separated},
    Solution,
};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::satisfy,
    combinator::{all_consuming, map},
    multi::separated_list1,
    sequence::{preceded, tuple},
    IResult,
};
use petgraph::{
    matrix_graph::{MatrixGraph, NodeIndex},
    visit::{IntoNodeIdentifiers, IntoNodeReferences},
    Undirected,
};
use std::hash::Hash;
use std::{cmp, default::Default};
use std::{cmp::Ordering, collections::HashMap};

#[derive(Default)]
pub struct Day16 {}

const PART_1_TIME_LIMIT: usize = 30;
const PART_2_TIME_LIMIT: usize = 26;
const VALVE_OPEN_COST: usize = 1;
const TUNNEL_MOVE_COST: usize = 1;

impl Solution for Day16 {
    type Part1Result = FlowUnit;
    type Part2Result = Self::Part1Result;

    type Input = Vec<Valve>;

    fn parse<'a>(
        &mut self,
        input: &'a str,
    ) -> Result<Self::Input, nom::Err<nom::error::Error<&'a str>>> {
        all_consuming(line_separated(valve))(input).map(|x| x.1)
    }

    fn run_part_1(&mut self, data: &Self::Input) -> Self::Part1Result {
        let (start, mut graph) = build_valve_graph(data);
        find_shortest_paths(&mut graph);
        remove_useless_nodes(&mut graph, start);

        let mut results: Vec<HashMap<State, FlowUnit>> = std::iter::repeat_with(HashMap::new)
            .take(PART_1_TIME_LIMIT + 1)
            .collect();
        let mut initial_state = State::default();
        initial_state.move_to(start);

        for target in graph.node_identifiers() {
            let dist = graph[(start, target)];

            if dist + VALVE_OPEN_COST <= PART_1_TIME_LIMIT {
                let mut new_state = initial_state;
                new_state.move_to(target);
                results[PART_1_TIME_LIMIT - dist].insert(new_state, 0);
            }
        }

        let mut result = 0;

        for time in (0..PART_1_TIME_LIMIT).rev() {
            let mut iterable = HashMap::new();
            std::mem::swap(&mut results[time], &mut iterable);
            for (state, total_flow) in iterable {
                let valve = state.current_valve;
                let gain = if time >= VALVE_OPEN_COST {
                    graph[valve] * ((time - VALVE_OPEN_COST) as FlowUnit)
                } else {
                    0
                };
                result = cmp::max(result, total_flow + gain);

                for target in graph.node_identifiers().filter(|&x| !state.is_visited(x)) {
                    let dist = graph[(valve, target)];

                    if dist + VALVE_OPEN_COST <= time {
                        let mut new_state = state;
                        new_state.move_to(target);
                        replace_if_better(
                            &mut results[time - dist - VALVE_OPEN_COST],
                            new_state,
                            total_flow + gain,
                        );
                        result = cmp::max(result, total_flow + gain);
                    }
                }
            }
        }

        result
    }

    fn run_part_2(&mut self, data: &Self::Input) -> Self::Part2Result {
        let (start, mut graph) = build_valve_graph(data);
        find_shortest_paths(&mut graph);
        remove_useless_nodes(&mut graph, start);

        let mut results: Vec<Vec<HashMap<DoubleState, FlowUnit>>> = std::iter::repeat_with(|| {
            std::iter::repeat_with(HashMap::new)
                .take(PART_2_TIME_LIMIT + 1)
                .collect()
        })
        .take(PART_2_TIME_LIMIT + 1)
        .collect();
        let mut initial_state = DoubleState::default();
        initial_state.move_1_to(start);
        initial_state.move_2_to(start);

        for target_1 in graph.node_identifiers() {
            for target_2 in graph.node_identifiers() {
                let dist_1 = graph[(start, target_1)];
                let dist_2 = graph[(start, target_2)];

                if dist_1 + VALVE_OPEN_COST <= PART_2_TIME_LIMIT
                    && dist_2 + VALVE_OPEN_COST <= PART_2_TIME_LIMIT
                {
                    let mut new_state = initial_state;
                    new_state.move_1_to(target_1);
                    new_state.move_2_to(target_2);
                    results[PART_2_TIME_LIMIT - dist_1][PART_2_TIME_LIMIT - dist_2]
                        .insert(new_state, 0);
                }
            }
        }

        let mut result = 0;

        for time_1 in (0..PART_2_TIME_LIMIT).rev() {
            for time_2 in (0..PART_2_TIME_LIMIT).rev() {
                let mut iterable = HashMap::new();
                std::mem::swap(&mut results[time_1][time_2], &mut iterable);
                for (state, total_flow) in iterable {
                    let valve_1 = state.current_valve_1;
                    let valve_2 = state.current_valve_2;
                    let gain_1 = if time_1 >= VALVE_OPEN_COST {
                        graph[valve_1] * ((time_1 - VALVE_OPEN_COST) as FlowUnit)
                    } else {
                        0
                    };
                    let gain_2 = if time_2 >= VALVE_OPEN_COST && (valve_1 != valve_2 || time_1 == 0)
                    {
                        graph[valve_2] * ((time_2 - VALVE_OPEN_COST) as FlowUnit)
                    } else {
                        0
                    };
                    result = cmp::max(result, total_flow + gain_1 + gain_2);

                    if state.visited_count() == graph.node_count() {
                        continue;
                    }

                    let unvisited: Vec<_> = graph
                        .node_identifiers()
                        .filter(|&x| !state.is_visited(x))
                        .collect();

                    for target_1 in unvisited.iter().copied() {
                        for target_2 in unvisited.iter().copied() {
                            let dist_1 = graph[(valve_1, target_1)];
                            let dist_2 = graph[(valve_2, target_2)];

                            if dist_1 + VALVE_OPEN_COST <= time_1 {
                                let mut new_state = state;
                                new_state.move_1_to(target_1);
                                replace_if_better(
                                    &mut results[time_1 - dist_1 - VALVE_OPEN_COST][0],
                                    new_state,
                                    total_flow + gain_1,
                                );
                                result = cmp::max(result, total_flow + gain_1);
                            }

                            if dist_2 + VALVE_OPEN_COST <= time_2 {
                                let mut new_state = state;
                                new_state.move_2_to(target_2);
                                replace_if_better(
                                    &mut results[0][time_2 - dist_2 - VALVE_OPEN_COST],
                                    new_state,
                                    total_flow + gain_2,
                                );
                                result = cmp::max(result, total_flow + gain_2);
                            }

                            if dist_1 + VALVE_OPEN_COST <= time_1
                                && dist_2 + VALVE_OPEN_COST <= time_2
                            {
                                let mut new_state = state;
                                new_state.move_1_to(target_1);
                                new_state.move_2_to(target_2);
                                replace_if_better(
                                    &mut results[time_1 - dist_1 - VALVE_OPEN_COST]
                                        [time_2 - dist_2 - VALVE_OPEN_COST],
                                    new_state,
                                    total_flow + gain_1 + gain_2,
                                );
                                result = cmp::max(result, total_flow + gain_1 + gain_2);
                            }
                        }
                    }
                }
            }
        }

        result
    }
}

fn replace_if_better<T: Eq + Hash, V: PartialOrd + Copy>(
    map: &mut HashMap<T, V>,
    key: T,
    value: V,
) {
    map.entry(key)
        .and_modify(|v| {
            if value.partial_cmp(v) == Some(Ordering::Greater) {
                *v = value;
            }
        })
        .or_insert(value);
}

fn build_valve_graph(valves: &[Valve]) -> (ValveIntId, ValveGraph) {
    let mut graph = ValveGraph::new_undirected();
    let mut valve_map = HashMap::new();

    for valve in valves.iter() {
        let node_idx = graph.add_node(valve.flow_rate);
        graph.add_edge(node_idx, node_idx, 0);
        valve_map.insert(valve.id, node_idx);
    }

    for valve in valves.iter() {
        let node_from = valve_map[&valve.id];

        for target in valve.tunnels.iter() {
            let node_to = valve_map[target];
            if !graph.has_edge(node_from, node_to) {
                graph.add_edge(node_from, node_to, TUNNEL_MOVE_COST);
            }
        }
    }

    (valve_map[&ValveId('A', 'A')], graph)
}

fn find_shortest_paths(graph: &mut ValveGraph) {
    let all_nodes: Vec<_> = graph.node_identifiers().collect();
    for node_through in all_nodes.iter().copied() {
        for node_from in all_nodes.iter().copied() {
            if !graph.has_edge(node_from, node_through) {
                continue;
            }
            for node_to in all_nodes.iter().copied() {
                if !graph.has_edge(node_to, node_through) {
                    continue;
                }
                let new_dist = graph[(node_from, node_through)] + graph[(node_through, node_to)];
                if graph.has_edge(node_from, node_to) {
                    let dist = graph.edge_weight_mut(node_from, node_to);
                    if new_dist < *dist {
                        *dist = new_dist;
                    }
                } else {
                    graph.add_edge(node_from, node_to, new_dist);
                }
            }
        }
    }
}

fn remove_useless_nodes(graph: &mut ValveGraph, start: ValveIntId) {
    for node in graph
        .node_references()
        .filter_map(|(id, &flow)| {
            if flow == 0 && id != start {
                Some(id)
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .into_iter()
    {
        graph.remove_node(node);
    }
}

type FlowUnit = u32;
type ValveGraph = MatrixGraph<FlowUnit, usize, Undirected>;
type ValveIntId = NodeIndex;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
struct State {
    current_valve: ValveIntId,
    visited: Bitmask,
}

#[derive(Debug, Default, Clone, Copy)]
struct DoubleState {
    current_valve_1: ValveIntId,
    current_valve_2: ValveIntId,
    visited: Bitmask,
}

impl PartialEq for DoubleState {
    fn eq(&self, other: &Self) -> bool {
        self.min() == other.min() && self.max() == other.max() && self.visited == other.visited
    }
}

impl Eq for DoubleState {}

impl Hash for DoubleState {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.min().hash(state);
        self.max().hash(state);
        self.visited.hash(state);
    }
}

pub struct Valve {
    id: ValveId,
    flow_rate: FlowUnit,
    tunnels: Vec<ValveId>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
struct Bitmask {
    mask: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct ValveId(char, char);

trait Statey {
    fn is_visited(&self, valve: ValveIntId) -> bool;

    fn visit(&mut self, valve: ValveIntId);

    fn visited_count(&self) -> usize;
}

impl Statey for State {
    fn is_visited(&self, valve: ValveIntId) -> bool {
        self.visited.is_set(valve.index())
    }

    fn visit(&mut self, valve: ValveIntId) {
        self.visited.set(valve.index())
    }

    fn visited_count(&self) -> usize {
        self.visited.len()
    }
}

impl State {
    fn move_to(&mut self, valve: ValveIntId) {
        self.current_valve = valve;
        self.visit(valve);
    }
}

impl Statey for DoubleState {
    fn is_visited(&self, valve: ValveIntId) -> bool {
        self.visited.is_set(valve.index())
    }

    fn visit(&mut self, valve: ValveIntId) {
        self.visited.set(valve.index())
    }

    fn visited_count(&self) -> usize {
        self.visited.len()
    }
}

impl DoubleState {
    fn move_1_to(&mut self, valve: ValveIntId) {
        self.current_valve_1 = valve;
        self.visited.set(valve.index());
    }

    fn move_2_to(&mut self, valve: ValveIntId) {
        self.current_valve_2 = valve;
        self.visited.set(valve.index());
    }

    fn min(&self) -> ValveIntId {
        cmp::min(self.current_valve_1, self.current_valve_2)
    }

    fn max(&self) -> ValveIntId {
        cmp::max(self.current_valve_1, self.current_valve_2)
    }
}

impl Bitmask {
    fn is_set(&self, id: usize) -> bool {
        (self.mask & (1 << id)) != 0
    }

    fn set(&mut self, id: usize) {
        self.mask |= 1 << id
    }

    fn len(&self) -> usize {
        self.mask.count_ones() as usize
    }
}

fn valve(input: &str) -> IResult<&str, Valve> {
    map(
        tuple((
            preceded(tag("Valve "), valve_id),
            preceded(tag(" has flow rate="), integer),
            preceded(
                alt((
                    tag("; tunnels lead to valves "),
                    tag("; tunnel leads to valve "),
                )),
                separated_list1(tag(", "), valve_id),
            ),
        )),
        |(id, flow_rate, tunnels)| Valve {
            id,
            flow_rate,
            tunnels,
        },
    )(input)
}

fn valve_id(input: &str) -> IResult<&str, ValveId> {
    map(
        tuple((
            satisfy(|c| c.is_ascii_uppercase()),
            satisfy(|c| c.is_ascii_uppercase()),
        )),
        |(a, b)| ValveId(a, b),
    )(input)
}
