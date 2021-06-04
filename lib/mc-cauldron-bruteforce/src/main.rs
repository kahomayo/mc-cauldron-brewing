use mc_cauldron_brew::BasicPotionIngredient::{
    BlazePowder, FermentedSpiderEye, GhastTear, MagmaCream, SpiderEye, Sugar,
};
use mc_cauldron_brew::{BasicPotionIngredient, LiquidData};
use std::collections::VecDeque;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::sync::atomic::Ordering::AcqRel;

#[derive(Eq, PartialEq, Copy, Clone, Hash, Debug)]
enum Action {
    Ingredient(BasicPotionIngredient),
    Dilute,
    Wart,
}

impl Action {
    pub fn apply_to(&self, ld: LiquidData) -> LiquidData {
        match self {
            Action::Ingredient(ing) => ld.apply_ingredient(*ing),
            Action::Dilute => ld.dilute(),
            Action::Wart => ld.apply_wart(),
        }
    }
}

const ACTIONS: [Action; 8] = [
    Action::Ingredient(Sugar),
    Action::Ingredient(GhastTear),
    Action::Ingredient(SpiderEye),
    Action::Ingredient(FermentedSpiderEye),
    Action::Ingredient(BlazePowder),
    Action::Ingredient(MagmaCream),
    Action::Dilute,
    Action::Wart,
];

fn main() -> Result<(), Box<dyn Error>> {
    let mut solutions: Vec<Option<Vec<Action>>> = vec![None; 32768];
    let mut queue: VecDeque<(Vec<Action>, LiquidData)> = VecDeque::new();

    solutions[LiquidData::default().0 as usize] = Some(Vec::new());
    queue.push_back((Vec::new(), LiquidData::default()));

    while !queue.is_empty() {
        let mut next_queue = VecDeque::new();
        for (prev_actions, prev_state) in queue.into_iter() {
            for action in ACTIONS.iter() {
                let state = action.apply_to(prev_state);
                if solutions[state.0 as usize].is_none() {
                    let mut actions = prev_actions.clone();
                    actions.push(*action);
                    next_queue.push_back((actions.clone(), state));
                    solutions[state.0 as usize] = Some(actions);
                }
            }
        }
        queue = next_queue;
    }

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
        "found {} solutions, at most {} long",
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
            Action::Ingredient(Sugar) => "S",
            Action::Ingredient(GhastTear) => "G",
            Action::Ingredient(SpiderEye) => "E",
            Action::Ingredient(FermentedSpiderEye) => "F",
            Action::Ingredient(BlazePowder) => "B",
            Action::Ingredient(MagmaCream) => "C",
            Action::Dilute => "W",
            Action::Wart => "N",
        })
        .collect();
    action_names.join("")
}
