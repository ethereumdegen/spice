use crate::agent::AgentOutput;
use std::ops::RangeInclusive;

/// Collect all tool names called within the specified turn range.
pub fn tools_in_range(output: &AgentOutput, range: &RangeInclusive<usize>) -> Vec<String> {
    output
        .turns
        .iter()
        .filter(|t| range.contains(&t.index))
        .flat_map(|t| t.tool_calls.iter().map(|tc| tc.name.clone()))
        .collect()
}

/// Find the first turn index where any of the specified tools appear.
pub fn first_turn_with_tools(output: &AgentOutput, tools: &[String]) -> Option<usize> {
    output.turns.iter().find_map(|t| {
        if t.tool_calls.iter().any(|tc| tools.contains(&tc.name)) {
            Some(t.index)
        } else {
            None
        }
    })
}
