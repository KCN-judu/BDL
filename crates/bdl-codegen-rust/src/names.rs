//! Stable generated symbols.  Every name is derived from a stable id —
//! never from a display name, which may change without changing meaning.
//! Display names appear in comments and in the manifest only.

use bdl_exec_ir::{ClockSlot, LocalId, StateSlot};
use bdl_model::{DeclId, OutputId, SemanticId};

pub fn concept(s: SemanticId) -> String {
    format!("Sem{}", s.raw())
}
pub fn decl(d: DeclId) -> String {
    format!("decl_{}", d.raw())
}
pub fn cell(s: StateSlot) -> String {
    format!("cell_{}", s.0)
}
/// The write-phase local holding a cell's new value this tick.
pub fn write(s: StateSlot) -> String {
    format!("write_{}", s.0)
}
pub fn output(o: OutputId) -> String {
    format!("output_{}", o.raw())
}
pub fn clock(c: ClockSlot) -> String {
    format!("CLOCK_{}", c.0)
}
pub fn local(l: LocalId) -> String {
    format!("l{}", l.0)
}

/// A Cargo package name for a design: `bdl_design_` plus the design name
/// reduced to `[a-z0-9_]`, never empty.
pub fn package(design: &str) -> String {
    let mut s = String::from("bdl_design_");
    let mut last_us = false;
    for ch in design.chars() {
        let c = ch.to_ascii_lowercase();
        if c.is_ascii_alphanumeric() {
            s.push(c);
            last_us = false;
        } else if !last_us {
            s.push('_');
            last_us = true;
        }
    }
    let trimmed = s.trim_end_matches('_').to_string();
    if trimmed == "bdl_design" {
        "bdl_design_unnamed".into()
    } else {
        trimmed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_names_are_valid_and_stable() {
        assert_eq!(package("lamp"), "bdl_design_lamp");
        assert_eq!(package("Robot Arm v2!"), "bdl_design_robot_arm_v2");
        assert_eq!(package("  "), "bdl_design_unnamed");
        assert_eq!(package("灯"), "bdl_design_unnamed");
    }

    #[test]
    fn symbols_come_from_ids_only() {
        assert_eq!(decl(DeclId::from_raw(17)), "decl_17");
        assert_eq!(concept(SemanticId::from_raw(3)), "Sem3");
        assert_eq!(cell(StateSlot(0)), "cell_0");
        assert_eq!(output(OutputId::from_raw(4)), "output_4");
        assert_eq!(clock(ClockSlot(2)), "CLOCK_2");
        assert_eq!(local(LocalId(9)), "l9");
    }
}
