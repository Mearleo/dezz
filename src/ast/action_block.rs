use std::{vec, collections::HashMap};

use crate::ast::{ Expression, BinaryOp };

pub fn resolve_action_block(args: Vec<Expression>, items: Vec<Expression>) -> Vec<Expression> {
    // vec of variables on the left side of Action
    let mut updated_vars: Vec<String> = Vec::new();
    // step: (updated variable)
    let mut steps: Vec<String> = Vec::new();

    // get all the updated variables
    for item in items.iter() {
        let Expression::Binary { op: BinaryOp::Action, left, .. } = item else {
            panic!("Expected Action expression inside action block, found {:?}", item);
        };

        let Expression::Ident(name) = left.as_ref() else {
            panic!("Left-hand side of Action must be an Ident, found {:?}", left);
        };

        // Track updated variables
        if !updated_vars.contains(name) {
            updated_vars.push(name.clone());
        }

        // Track the step
        steps.push(name.clone());
    }

    let mut expressions = vec![create_first(&updated_vars, &steps, &args)];
    expressions.append(&mut create_helpers(&updated_vars, &steps, &items, &args));

    expressions

}

// create the action collection
fn create_first(updated_vars: &Vec<String>, steps: &Vec<String>, block_args: &Vec<Expression>) -> Expression {
    let actions = 
        updated_vars.iter()
        .map(|var| 
            {
                let left = Expression::Ident(var.to_string());

                let last_update = get_last_update(var.clone(), &steps).unwrap();
                let name = format!("{}{}", last_update, var);

                // arguments will be (..block_args, ..temp_args)
                let mut args = block_args.to_vec();
                args.extend(updated_vars.iter().map(|var| Expression::Ident(var.to_string())));

                let right = Expression::Call { ident: name, args };

                Expression::Binary { op: BinaryOp::Action, left: Box::new(left), right: Box::new(right)}
            }
        )
        .collect();

    Expression::ActionCollection(actions)
}

fn helper_name(line: usize, name: &String) -> String {
    return format!("{}{}", line, name);
}

// create the helper functions
fn create_helpers(updated_vars: &Vec<String>, steps: &Vec<String>, items: &Vec<Expression>, block_args: &Vec<Expression>) -> Vec<Expression> {
    let mut helpers: Vec<Expression> = Vec::new();

    for (i, var) in steps.iter().enumerate() {
        let item = items[i].clone();

        let name = helper_name(i, var);

        let mut args = block_args.to_vec();
        args.extend(updated_vars.iter().map(|var| Expression::Ident(var.to_string())));

        let left = Expression::Call { ident: name, args };

        // Extract the right-hand side of an Action expression and rewrite it
        let right = match item {
            Expression::Binary { op: BinaryOp::Action, right, .. } => {
                // get the variables needed to be replaced
                let mut vars: Vec<String> = Vec::new();

                for updated in updated_vars.iter() {
                    if get_first_update(updated.clone(), steps).unwrap() < i {
                        vars.push(updated.clone());
                    }
                }

                let mut args = block_args.to_vec();
                args.extend(updated_vars.iter().map(|var| Expression::Ident(var.clone())));

                let tuples: Vec<(String, Expression)> = vars.iter().map(|var| {
                    let last_up = get_last_update_before(var.to_string(), i, steps);
                    let call_name = helper_name(last_up.unwrap(), &var);

                    (
                        var.clone(),
                        Expression::Call { ident: call_name, args: args.clone() }
                    )
                }).collect();

                let replacements: HashMap<String, Expression> = tuples.into_iter().collect();


                // Rewrite
                let mut rewritten = right.as_ref().clone();

                rewritten.walk_mut(&mut |node| {
                    if let Expression::Ident(name) = node {
                        if let Some(replacement) = replacements.get(name) {
                            *node = replacement.clone();
                            return false; // stop walking inside replacement
                        }
                    }
                    true // continue walking normally
                });

                rewritten
            }

            Expression::Binary { op, .. } => {
                panic!("Expected Action, found operator {:?}.", op);
            }

            other => {
                panic!("Expected Binary(Action), found {:?}.", other);
            }
        };

        helpers.push(Expression::Binary {
            op: BinaryOp::Eq,
            left: Box::new(left),
            right: Box::new(right),
        });
    }

    helpers
}

// returns index of the first step it got updated
fn get_first_update(name: String, steps: &Vec<String>) -> Option<usize> {
    let mut pos = 0;
    while steps[pos] != name {
        if pos == steps.len() - 1 {
            return None
        }
        pos += 1;
    }
    Some(pos)
}

fn get_last_update_before(name: String, i: usize, steps: &Vec<String>) -> Option<usize> {
    let mut pos = 0;
    let mut seen: bool = false;
    let mut last_seen: usize = 0;
    while pos < i {
        if steps[pos] == name {
            last_seen = pos;
            seen = true;
        }
        pos += 1;
    }
    if seen {
        Some(last_seen)
    } else {
        None
    }
}

fn get_last_update(name: String, steps: &Vec<String>) -> Option<usize> {
    let mut pos = 0;
    let mut seen: bool = false;
    let mut last_seen: usize = 0;
    while pos < steps.len() {
        if steps[pos] == name {
            last_seen = pos;
            seen = true;
        }
        pos += 1;
    }
    if seen {
        Some(last_seen)
    } else {
        None
    }
}