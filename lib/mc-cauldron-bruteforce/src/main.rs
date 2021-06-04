use mc_cauldron_brew::PotionIngredient::{
    BlazePowder, FermentedSpiderEye, GhastTear, MagmaCream, SpiderEye, Sugar,
};
use mc_cauldron_brew::{LiquidData, PotionIngredient};
use std::collections::VecDeque;
use std::error::Error;
use std::fs::File;
use std::io::Write;

/// Represents one interaction with a cauldron
#[derive(Eq, PartialEq, Copy, Clone, Hash, Debug)]
enum Action {
    AddIngredient(PotionIngredient),
    Dilute,
    AddNetherWart,
}

fn main() -> Result<(), Box<dyn Error>> {
    // solutions[dv] = actions to produce potion with that dv.
    let mut solutions: Vec<Option<Vec<Action>>> = vec![None; 32768];
    let mut queue: VecDeque<(Vec<Action>, LiquidData)> = VecDeque::new();

    // Add the starting potion (plain water)
    solutions[LiquidData::default().0 as usize] = Some(Vec::new());
    queue.push_back((Vec::new(), LiquidData::default()));

    // Perform a BFS (breadth-first search)
    while !queue.is_empty() {
        let mut next_queue = VecDeque::new();
        // for every state in the queue
        for (prev_actions, prev_state) in queue.into_iter() {
            // check all possible actions to take from there
            for action in ALL_ACTIONS.iter() {
                let state = action.apply_to(prev_state);
                // if that action leads to a new potion
                if solutions[state.0 as usize].is_none() {
                    // save the steps to get there and add it to the next queue
                    let mut actions = prev_actions.clone();
                    actions.push(*action);
                    next_queue.push_back((actions.clone(), state));
                    solutions[state.0 as usize] = Some(actions);
                }
            }
        }
        queue = next_queue;
    }

    // Write results to some file
    let mut writer = File::create("results.txt")?;
    // for (i, actions) in solutions.iter().enumerate() {
    //     let actions = actions
    //         .as_ref()
    //         .map(|actions| format_actions(actions))
    //         .unwrap_or("------".to_string());
    //     writeln!(writer, "{:05}, {}", i, actions);
    // }
    for (i, actions) in solutions
        .iter()
        .enumerate()
        .filter_map(|(i, a)| a.as_ref().map(|a| (i, a)))
    {
        writeln!(writer, "{:05}, {}", i, format_actions(actions))?;
    }

    writer.sync_all()?;
    println!(
        "found {} solutions, at most {} steps long",
        solutions.iter().filter(|s| s.is_some()).count(),
        solutions
            .iter()
            .filter_map(|s| s.as_ref())
            .map(|s| s.len())
            .max()
            .expect("There should be some answers"),
    );
    Ok(())
}

fn format_actions(actions: &Vec<Action>) -> String {
    let action_names: Vec<_> = actions
        .iter()
        .map(|a| match a {
            Action::AddIngredient(Sugar) => "S",
            Action::AddIngredient(GhastTear) => "G",
            Action::AddIngredient(SpiderEye) => "E",
            Action::AddIngredient(FermentedSpiderEye) => "F",
            Action::AddIngredient(BlazePowder) => "B",
            Action::AddIngredient(MagmaCream) => "C",
            Action::Dilute => "W",
            Action::AddNetherWart => "N",
        })
        .collect();
    action_names.join("")
}

const ALL_ACTIONS: [Action; 8] = [
    Action::AddIngredient(Sugar),
    Action::AddIngredient(GhastTear),
    Action::AddIngredient(SpiderEye),
    Action::AddIngredient(FermentedSpiderEye),
    Action::AddIngredient(BlazePowder),
    Action::AddIngredient(MagmaCream),
    Action::Dilute,
    Action::AddNetherWart,
];

impl Action {
    pub fn apply_to(&self, ld: LiquidData) -> LiquidData {
        match self {
            Action::AddIngredient(ing) => ld.apply_ingredient(*ing),
            Action::Dilute => ld.dilute(),
            Action::AddNetherWart => ld.apply_wart(),
        }
    }
}
