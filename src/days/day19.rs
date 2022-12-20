use crate::{
    parsing::{integer, line_separated},
    Solution,
};
use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{all_consuming, map, opt},
    sequence::{delimited, preceded, terminated, tuple},
    IResult,
};
use ordered_float::NotNan;
use rayon::prelude::*;
use std::{
    cmp::{self, Ordering},
    collections::HashSet,
};

#[derive(Default)]
pub struct Day19 {}

const ALL_BUILD_ACTIONS: [Action; 4] = [
    Action::BuildGeodeRobot,
    Action::BuildObsidianRobot,
    Action::BuildClayRobot,
    Action::BuildOreRobot,
];

impl Solution for Day19 {
    type Part1Result = usize;
    type Part2Result = Self::Part1Result;

    type Input = Vec<Blueprint>;

    fn parse<'a>(
        &mut self,
        input: &'a str,
    ) -> Result<Self::Input, nom::Err<nom::error::Error<&'a str>>> {
        all_consuming(line_separated(blueprint))(input).map(|x| x.1)
    }

    fn run_part_1(&mut self, data: &Self::Input) -> Self::Part1Result {
        data.par_iter().map(|b| b.id * run_blueprint(b, 24)).sum()
    }

    fn run_part_2(&mut self, data: &Self::Input) -> Self::Part2Result {
        data.par_iter()
            .take(3)
            .map(|b| run_blueprint(b, 32))
            .product()
    }
}

fn run_blueprint(blueprint: &Blueprint, time_limit: usize) -> usize {
    let start = State::default();
    exhaustive_search(blueprint, start, time_limit)
}

fn exhaustive_search(blueprint: &Blueprint, start: State, time_limit: usize) -> usize {
    let mut active = Vec::new();
    let mut max_geodes = 0;
    let mut visited = HashSet::new();

    active.push(start);

    while let Some(state) = active.pop() {
        if !visited.insert(state) || state.estimate_max_geodes(time_limit) <= max_geodes {
            continue;
        }

        max_geodes = cmp::max(state.resources.geode, max_geodes);
        let mut any_failed = false;

        for action in ALL_BUILD_ACTIONS {
            match state.next_state(action, blueprint, time_limit) {
                ActionResult::Possible(next_state) => active.push(next_state),
                ActionResult::NotEnoughResources => any_failed = true,
                ActionResult::Useless => (),
            }
        }

        if any_failed {
            if let ActionResult::Possible(next_state) =
                state.next_state(Action::DoNothing, blueprint, time_limit)
            {
                active.push(next_state);
            }
        }
    }

    max_geodes
}

type Float = NotNan<f32>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct GeodeEst(Float);

impl PartialOrd for GeodeEst {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0).map(|x| x.reverse())
    }
}

impl Ord for GeodeEst {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0).reverse()
    }
}

#[derive(Debug)]
pub struct Blueprint {
    id: usize,
    ore_robot_cost: Cost,
    clay_robot_cost: Cost,
    obsidian_robot_cost: Cost,
    geode_robot_cost: Cost,
    max_ore_cost: usize,
    max_clay_cost: usize,
    max_obsidian_cost: usize,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
struct Cost {
    ore: usize,
    clay: usize,
    obsidian: usize,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct State {
    time_elapsed: usize,
    robots: Robots,
    resources: Resources,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Robots {
    ore: usize,
    clay: usize,
    obsidian: usize,
    geode: usize,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
struct Resources {
    ore: usize,
    clay: usize,
    obsidian: usize,
    geode: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Action {
    DoNothing,
    BuildOreRobot,
    BuildClayRobot,
    BuildObsidianRobot,
    BuildGeodeRobot,
}

enum ActionResult {
    NotEnoughResources,
    Useless,
    Possible(State),
}

impl State {
    fn next_state(&self, action: Action, blueprint: &Blueprint, time_limit: usize) -> ActionResult {
        let action_cost = match action {
            Action::BuildOreRobot if self.robots.ore >= blueprint.max_ore_cost => {
                return ActionResult::Useless
            }
            Action::BuildClayRobot if self.robots.clay >= blueprint.max_clay_cost => {
                return ActionResult::Useless
            }
            Action::BuildObsidianRobot if self.robots.obsidian >= blueprint.max_obsidian_cost => {
                return ActionResult::Useless
            }
            Action::DoNothing
                if self.resources.ore >= blueprint.max_ore_cost
                    && self.resources.clay >= blueprint.max_clay_cost
                    && self.resources.obsidian >= blueprint.max_obsidian_cost =>
            {
                return ActionResult::Useless
            }
            Action::BuildOreRobot => blueprint.ore_robot_cost,
            Action::BuildClayRobot => blueprint.clay_robot_cost,
            Action::BuildObsidianRobot => blueprint.obsidian_robot_cost,
            Action::BuildGeodeRobot => blueprint.geode_robot_cost,
            Action::DoNothing => Cost::default(),
        };

        if self.time_elapsed == time_limit
            || self.resources.ore < action_cost.ore
            || self.resources.clay < action_cost.clay
            || self.resources.obsidian < action_cost.obsidian
        {
            return ActionResult::NotEnoughResources;
        }

        let resources = Resources {
            ore: self.resources.ore + self.robots.ore - action_cost.ore,
            clay: self.resources.clay + self.robots.clay - action_cost.clay,
            obsidian: self.resources.obsidian + self.robots.obsidian - action_cost.obsidian,
            geode: self.resources.geode + self.robots.geode,
        };
        let robots = match action {
            Action::DoNothing => self.robots,
            Action::BuildOreRobot => Robots {
                ore: self.robots.ore + 1,
                ..self.robots
            },
            Action::BuildClayRobot => Robots {
                clay: self.robots.clay + 1,
                ..self.robots
            },
            Action::BuildObsidianRobot => Robots {
                obsidian: self.robots.obsidian + 1,
                ..self.robots
            },
            Action::BuildGeodeRobot => Robots {
                geode: self.robots.geode + 1,
                ..self.robots
            },
        };

        ActionResult::Possible(State {
            time_elapsed: self.time_elapsed + 1,
            resources,
            robots,
        })
    }

    fn estimate_max_geodes(&self, time_limit: usize) -> usize {
        let time = time_limit - self.time_elapsed;
        self.resources.geode + time * self.robots.geode + (time * (time - 1)) / 2
    }
}

impl Default for Robots {
    fn default() -> Self {
        Self {
            ore: 1,
            clay: 0,
            obsidian: 0,
            geode: 0,
        }
    }
}

impl PartialOrd for Robots {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (
            self.ore.cmp(&other.ore),
            self.clay.cmp(&other.clay),
            self.obsidian.cmp(&other.obsidian),
            self.geode.cmp(&other.geode),
        ) {
            (Ordering::Less, Ordering::Less, Ordering::Less, Ordering::Less) => {
                Some(Ordering::Less)
            }
            (Ordering::Equal, Ordering::Equal, Ordering::Equal, Ordering::Equal) => {
                Some(Ordering::Equal)
            }
            (Ordering::Greater, Ordering::Greater, Ordering::Greater, Ordering::Greater) => {
                Some(Ordering::Greater)
            }
            _ => None,
        }
    }
}

impl Ord for Robots {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for Resources {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (
            self.ore.cmp(&other.ore),
            self.clay.cmp(&other.clay),
            self.obsidian.cmp(&other.obsidian),
            self.geode.cmp(&other.geode),
        ) {
            (Ordering::Less, Ordering::Less, Ordering::Less, Ordering::Less) => {
                Some(Ordering::Less)
            }
            (Ordering::Equal, Ordering::Equal, Ordering::Equal, Ordering::Equal) => {
                Some(Ordering::Equal)
            }
            (Ordering::Greater, Ordering::Greater, Ordering::Greater, Ordering::Greater) => {
                Some(Ordering::Greater)
            }
            _ => None,
        }
    }
}

impl Ord for Resources {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

impl Blueprint {
    fn new(
        id: usize,
        ore_robot_cost: Cost,
        clay_robot_cost: Cost,
        obsidian_robot_cost: Cost,
        geode_robot_cost: Cost,
    ) -> Self {
        Self {
            id,
            ore_robot_cost,
            clay_robot_cost,
            obsidian_robot_cost,
            geode_robot_cost,
            max_ore_cost: cmp::max(
                cmp::max(ore_robot_cost.ore, clay_robot_cost.ore),
                cmp::max(obsidian_robot_cost.ore, geode_robot_cost.ore),
            ),
            max_clay_cost: cmp::max(
                cmp::max(ore_robot_cost.clay, clay_robot_cost.clay),
                cmp::max(obsidian_robot_cost.clay, geode_robot_cost.clay),
            ),
            max_obsidian_cost: cmp::max(
                cmp::max(ore_robot_cost.obsidian, clay_robot_cost.obsidian),
                cmp::max(obsidian_robot_cost.obsidian, geode_robot_cost.obsidian),
            ),
        }
    }
}

fn blueprint(input: &str) -> IResult<&str, Blueprint> {
    map(
        tuple((
            preceded(tag("Blueprint "), integer),
            preceded(tag(": Each ore robot costs "), cost),
            preceded(tag(". Each clay robot costs "), cost),
            preceded(tag(". Each obsidian robot costs "), cost),
            delimited(tag(". Each geode robot costs "), cost, tag(".")),
        )),
        |(id, ore_robot_cost, clay_robot_cost, obsidian_robot_cost, geode_robot_cost)| {
            Blueprint::new(
                id,
                ore_robot_cost,
                clay_robot_cost,
                obsidian_robot_cost,
                geode_robot_cost,
            )
        },
    )(input)
}

fn cost(input: &str) -> IResult<&str, Cost> {
    map(
        tuple((cost_unit, opt(preceded(tag(" and "), cost_unit)))),
        |(x, opt_y)| Cost {
            ore: x.0 + opt_y.map_or(0, |(y, _, _)| y),
            clay: x.1 + opt_y.map_or(0, |(_, y, _)| y),
            obsidian: x.2 + opt_y.map_or(0, |(_, _, y)| y),
        },
    )(input)
}

fn cost_unit(input: &str) -> IResult<&str, (usize, usize, usize)> {
    alt((
        map(terminated(integer, tag(" ore")), |x| (x, 0, 0)),
        map(terminated(integer, tag(" clay")), |x| (0, x, 0)),
        map(terminated(integer, tag(" obsidian")), |x| (0, 0, x)),
    ))(input)
}
