//! Behaviour grouping — authoring metadata over the base design's
//! relationships (FV Phase 8b `Group.lean`, `BEHAVIOR_GROUPING_NOTE.md`).
//!
//! A group is an identity, a name, a description and a member list.  Every
//! operation here changes the group table only: the base design, the
//! components, the instances and the bindings are untouched, so the
//! flattening, every analysis, every simulation trace and every deployment
//! verdict are *literally* those of the ungrouped system (Theorems A–G,
//! all `rfl`).  Hence no group edit creates a revision, invalidates a
//! cache, or resets a simulation: [`GroupEditOutcome`] carries no
//! invalidation set at all, and the daemon applies these outside the
//! revision stream, on an *authoring generation* of its own.
//!
//! A group lives in one authored design — the system's own, or a
//! component's body (`GroupScope`) — and its members are relationships of
//! that design; a relationship is in at most one group of its scope (a
//! region on the canvas contains a node once).  The FV allows overlapping
//! lists and states its results for any design; the one-group restriction
//! is an authoring choice (DI-36) that costs no theorem, and the explicit
//! scope is how production names the design (never inferred from ids,
//! DI-42).  A group never spans two designs: moving a relationship across
//! a component boundary is an extraction or a body edit, not grouping.

use crate::ids::{BehaviorGroupId, ComponentId};
use crate::model::*;
use bdl_model::DeclId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "edit", rename_all = "snake_case")]
pub enum GroupEditOp {
    /// FV `group`: a fresh group over some relationships of one design.
    CreateGroup {
        #[serde(default)]
        scope: GroupScope,
        name: String,
        #[serde(default)]
        description: String,
        #[serde(default)]
        members: Vec<DeclId>,
    },
    RenameGroup {
        id: BehaviorGroupId,
        name: String,
    },
    SetGroupDescription {
        id: BehaviorGroupId,
        description: String,
    },
    /// FV `ungroup`: the group goes, its members stay where they are.
    DeleteGroup {
        id: BehaviorGroupId,
    },
    AddMember {
        group: BehaviorGroupId,
        decl: DeclId,
    },
    RemoveMember {
        group: BehaviorGroupId,
        decl: DeclId,
    },
    /// FV `move`: from whichever group holds it (or none) into `to`.
    MoveMember {
        decl: DeclId,
        to: BehaviorGroupId,
    },
    /// FV `merge`: `from`'s members join `into`; `from` goes.
    MergeGroups {
        into: BehaviorGroupId,
        from: BehaviorGroupId,
    },
    /// FV `split`: the listed members leave `id` for a fresh group.
    SplitGroup {
        id: BehaviorGroupId,
        name: String,
        members: Vec<DeclId>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error, Serialize, Deserialize)]
#[serde(tag = "refused", rename_all = "snake_case")]
pub enum GroupEditError {
    #[error("a name is required")]
    EmptyName,
    #[error("a group named `{name}` already exists")]
    DuplicateGroupName { name: String },
    #[error("unknown group {id}")]
    UnknownGroup { id: BehaviorGroupId },
    #[error("declaration {decl} is not a relationship of the system's own design")]
    NotABaseDeclaration { decl: DeclId },
    #[error("declaration {decl} already belongs to group {group}")]
    AlreadyGrouped {
        decl: DeclId,
        group: BehaviorGroupId,
    },
    #[error("declaration {decl} is not a member of group {group}")]
    NotAMember {
        group: BehaviorGroupId,
        decl: DeclId,
    },
    #[error("a group cannot be merged into itself")]
    SameGroup,
    #[error("the groups belong to different designs")]
    ScopeMismatch,
    #[error("unknown component {id}")]
    UnknownComponent { id: ComponentId },
}

/// What a group edit did.  Deliberately no `invalidates`: nothing
/// semantic can change (Theorem A).
#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct GroupEditOutcome {
    pub created_group: Option<BehaviorGroupId>,
    /// Groups whose membership, name or description changed, or that were
    /// removed.
    pub groups: BTreeSet<BehaviorGroupId>,
}

fn valid_name(name: &str) -> Result<String, GroupEditError> {
    let n = name.trim();
    if n.is_empty() {
        Err(GroupEditError::EmptyName)
    } else {
        Ok(n.to_owned())
    }
}

/// Names are unique within a scope (two components may each have a
/// "Dimming").
fn unique_name(
    s: &BehaviorSystem,
    scope: GroupScope,
    name: String,
    except: Option<BehaviorGroupId>,
) -> Result<String, GroupEditError> {
    if s.groups_in(scope)
        .any(|g| g.name == name && Some(g.id) != except)
    {
        return Err(GroupEditError::DuplicateGroupName { name });
    }
    Ok(name)
}

fn scope_exists(s: &BehaviorSystem, scope: GroupScope) -> Result<(), GroupEditError> {
    match scope {
        GroupScope::SystemBase => Ok(()),
        GroupScope::Component { component } if s.components.contains_key(&component) => Ok(()),
        GroupScope::Component { component } => {
            Err(GroupEditError::UnknownComponent { id: component })
        }
    }
}

fn group_mut(
    s: &mut BehaviorSystem,
    id: BehaviorGroupId,
) -> Result<&mut BehaviorGroup, GroupEditError> {
    s.groups
        .get_mut(&id)
        .ok_or(GroupEditError::UnknownGroup { id })
}

/// `decl` is a relationship of the scope's design.
fn scoped_decl(
    s: &BehaviorSystem,
    scope: GroupScope,
    decl: DeclId,
) -> Result<DeclId, GroupEditError> {
    scope_exists(s, scope)?;
    let design = s
        .design_of(scope)
        .ok_or(GroupEditError::NotABaseDeclaration { decl })?;
    if design.mappings.contains_key(&decl) {
        Ok(decl)
    } else {
        Err(GroupEditError::NotABaseDeclaration { decl })
    }
}

/// `decl` may join `group`: it is a relationship of the scope's design in
/// no other group of that scope.
fn free_for(
    s: &BehaviorSystem,
    scope: GroupScope,
    decl: DeclId,
    group: Option<BehaviorGroupId>,
) -> Result<(), GroupEditError> {
    scoped_decl(s, scope, decl)?;
    if let Some(g) = s.group_of(scope, decl) {
        if Some(g.id) != group {
            return Err(GroupEditError::AlreadyGrouped { decl, group: g.id });
        }
    }
    Ok(())
}

/// Apply one group edit.  Pure; the base design is read (membership must
/// name its relationships) and never written.
pub fn apply_group_edit(
    system: &BehaviorSystem,
    op: &GroupEditOp,
) -> Result<(BehaviorSystem, GroupEditOutcome), GroupEditError> {
    let mut s = system.clone();
    let mut o = GroupEditOutcome::default();
    match op {
        GroupEditOp::CreateGroup {
            scope,
            name,
            description,
            members,
        } => {
            scope_exists(&s, *scope)?;
            let name = unique_name(&s, *scope, valid_name(name)?, None)?;
            let mut list = Vec::new();
            for d in members {
                free_for(&s, *scope, *d, None)?;
                if !list.contains(d) {
                    list.push(*d);
                }
            }
            let id = s.ids.fresh_group();
            s.groups.insert(
                id,
                BehaviorGroup {
                    id,
                    scope: *scope,
                    name,
                    description: description.clone(),
                    members: list,
                },
            );
            o.created_group = Some(id);
            o.groups.insert(id);
        }
        GroupEditOp::RenameGroup { id, name } => {
            let scope = group_mut(&mut s, *id)?.scope;
            let name = unique_name(&s, scope, valid_name(name)?, Some(*id))?;
            group_mut(&mut s, *id)?.name = name;
            o.groups.insert(*id);
        }
        GroupEditOp::SetGroupDescription { id, description } => {
            group_mut(&mut s, *id)?.description = description.clone();
            o.groups.insert(*id);
        }
        GroupEditOp::DeleteGroup { id } => {
            s.groups
                .remove(id)
                .ok_or(GroupEditError::UnknownGroup { id: *id })?;
            o.groups.insert(*id);
        }
        GroupEditOp::AddMember { group, decl } => {
            let scope = group_mut(&mut s, *group)?.scope;
            free_for(&s, scope, *decl, Some(*group))?;
            let g = group_mut(&mut s, *group)?;
            if !g.members.contains(decl) {
                g.members.push(*decl);
            }
            o.groups.insert(*group);
        }
        GroupEditOp::RemoveMember { group, decl } => {
            let g = group_mut(&mut s, *group)?;
            let Some(i) = g.members.iter().position(|m| m == decl) else {
                return Err(GroupEditError::NotAMember {
                    group: *group,
                    decl: *decl,
                });
            };
            g.members.remove(i);
            o.groups.insert(*group);
        }
        GroupEditOp::MoveMember { decl, to } => {
            let scope = group_mut(&mut s, *to)?.scope;
            scoped_decl(&s, scope, *decl)?;
            if let Some(from) = s.group_of(scope, *decl).map(|g| g.id) {
                if from == *to {
                    return Ok((s, o));
                }
                let g = group_mut(&mut s, from)?;
                g.members.retain(|m| m != decl);
                o.groups.insert(from);
            }
            group_mut(&mut s, *to)?.members.push(*decl);
            o.groups.insert(*to);
        }
        GroupEditOp::MergeGroups { into, from } => {
            if into == from {
                return Err(GroupEditError::SameGroup);
            }
            let into_scope = group_mut(&mut s, *into)?.scope;
            let from_scope = group_mut(&mut s, *from)?.scope;
            if into_scope != from_scope {
                return Err(GroupEditError::ScopeMismatch);
            }
            let moved = s
                .groups
                .remove(from)
                .ok_or(GroupEditError::UnknownGroup { id: *from })?;
            let g = group_mut(&mut s, *into)?;
            for d in moved.members {
                if !g.members.contains(&d) {
                    g.members.push(d);
                }
            }
            o.groups.insert(*into);
            o.groups.insert(*from);
        }
        GroupEditOp::SplitGroup { id, name, members } => {
            let scope = group_mut(&mut s, *id)?.scope;
            let name = unique_name(&s, scope, valid_name(name)?, None)?;
            let g = group_mut(&mut s, *id)?;
            for d in members {
                if !g.members.contains(d) {
                    return Err(GroupEditError::NotAMember {
                        group: *id,
                        decl: *d,
                    });
                }
            }
            let moved: Vec<DeclId> = g
                .members
                .iter()
                .copied()
                .filter(|m| members.contains(m))
                .collect();
            g.members.retain(|m| !members.contains(m));
            let fresh = s.ids.fresh_group();
            s.groups.insert(
                fresh,
                BehaviorGroup {
                    id: fresh,
                    scope,
                    name,
                    description: String::new(),
                    members: moved,
                },
            );
            o.created_group = Some(fresh);
            o.groups.insert(*id);
            o.groups.insert(fresh);
        }
    }
    Ok((s, o))
}

/// After a semantic edit: a member whose relationship is gone leaves its
/// group (the group itself stays, possibly empty — the designer's unit
/// outlives one deletion); the groups of a component that is gone go with
/// it (no orphan scope).
pub fn prune_groups(s: &mut BehaviorSystem) {
    let scopes: Vec<(BehaviorGroupId, GroupScope)> =
        s.groups.values().map(|g| (g.id, g.scope)).collect();
    for (id, scope) in scopes {
        let Some(design) = s.design_of(scope) else {
            s.groups.remove(&id);
            continue;
        };
        let live: Vec<DeclId> = design.mappings.keys().copied().collect();
        if let Some(g) = s.groups.get_mut(&id) {
            g.members.retain(|d| live.contains(d));
        }
    }
}

/// A component's groups copied for a version of it: fresh group ids, the
/// same local member ids (a version keeps its local ids).  Group ids are
/// never shared between components.
pub fn copy_groups(s: &mut BehaviorSystem, from: ComponentId, to: ComponentId) {
    let originals: Vec<BehaviorGroup> = s
        .groups_in(GroupScope::Component { component: from })
        .cloned()
        .collect();
    for g in originals {
        let id = s.ids.fresh_group();
        s.groups.insert(
            id,
            BehaviorGroup {
                id,
                scope: GroupScope::Component { component: to },
                name: g.name,
                description: g.description,
                members: g.members,
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::ComponentId;

    #[test]
    fn a_group_without_a_scope_reads_as_base_scoped_and_scopes_round_trip() {
        let legacy = r#"{"id":0,"name":"Lamp","members":[1,2]}"#;
        let g: BehaviorGroup = serde_json::from_str(legacy).unwrap();
        assert_eq!(g.scope, GroupScope::SystemBase);
        let scoped = BehaviorGroup {
            id: BehaviorGroupId::from_raw(1),
            scope: GroupScope::Component {
                component: ComponentId::from_raw(3),
            },
            name: "Dimming".into(),
            description: String::new(),
            members: vec![DeclId::from_raw(4)],
        };
        let json = serde_json::to_string(&scoped).unwrap();
        assert!(
            json.contains(r#""scope":{"kind":"component","component":3}"#),
            "{json}"
        );
        let back: BehaviorGroup = serde_json::from_str(&json).unwrap();
        assert_eq!(back, scoped);
        let base_json = serde_json::to_string(&g).unwrap();
        assert!(
            base_json.contains(r#""scope":{"kind":"system_base"}"#),
            "{base_json}"
        );
    }
}
