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
//! Members are relationships of `base`; a relationship is in at most one
//! group (a region on the canvas contains a node once).  The FV allows
//! overlapping lists; the restriction is an authoring choice (DI-36) that
//! costs no theorem.

use crate::ids::BehaviorGroupId;
use crate::model::*;
use bdl_model::DeclId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "edit", rename_all = "snake_case")]
pub enum GroupEditOp {
    /// FV `group`: a fresh group over some relationships.
    CreateGroup {
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

fn unique_name(
    s: &BehaviorSystem,
    name: String,
    except: Option<BehaviorGroupId>,
) -> Result<String, GroupEditError> {
    if s.groups
        .values()
        .any(|g| g.name == name && Some(g.id) != except)
    {
        return Err(GroupEditError::DuplicateGroupName { name });
    }
    Ok(name)
}

fn group_mut(
    s: &mut BehaviorSystem,
    id: BehaviorGroupId,
) -> Result<&mut BehaviorGroup, GroupEditError> {
    s.groups
        .get_mut(&id)
        .ok_or(GroupEditError::UnknownGroup { id })
}

fn base_decl(s: &BehaviorSystem, decl: DeclId) -> Result<DeclId, GroupEditError> {
    if s.base.mappings.contains_key(&decl) {
        Ok(decl)
    } else {
        Err(GroupEditError::NotABaseDeclaration { decl })
    }
}

/// `decl` may join `group`: it is a base relationship in no other group.
fn free_for(
    s: &BehaviorSystem,
    decl: DeclId,
    group: Option<BehaviorGroupId>,
) -> Result<(), GroupEditError> {
    base_decl(s, decl)?;
    if let Some(g) = s.group_of(decl) {
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
            name,
            description,
            members,
        } => {
            let name = unique_name(&s, valid_name(name)?, None)?;
            let mut list = Vec::new();
            for d in members {
                free_for(&s, *d, None)?;
                if !list.contains(d) {
                    list.push(*d);
                }
            }
            let id = s.ids.fresh_group();
            s.groups.insert(
                id,
                BehaviorGroup {
                    id,
                    name,
                    description: description.clone(),
                    members: list,
                },
            );
            o.created_group = Some(id);
            o.groups.insert(id);
        }
        GroupEditOp::RenameGroup { id, name } => {
            let name = unique_name(&s, valid_name(name)?, Some(*id))?;
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
            group_mut(&mut s, *group)?;
            free_for(&s, *decl, Some(*group))?;
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
            group_mut(&mut s, *to)?;
            base_decl(&s, *decl)?;
            if let Some(from) = s.group_of(*decl).map(|g| g.id) {
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
            group_mut(&mut s, *into)?;
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
            let name = unique_name(&s, valid_name(name)?, None)?;
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
/// outlives one deletion).
pub fn prune_groups(s: &mut BehaviorSystem) {
    let base = &s.base;
    for g in s.groups.values_mut() {
        g.members.retain(|d| base.mappings.contains_key(d));
    }
}
