// Dropoff Wave Front Algorithm
use crate::{Result, error_msg};
use super::Penalties;
use super::Sequence;
use super::{AlignmentOperation, AlignmentType};
use super::{Extension, OperationsOfExtension, OwnedOperations, RefToOperations, StartPointOfOperations};

type MatchCounter<'a> = &'a dyn Fn(Sequence, Sequence, usize, usize) -> i32;

use std::collections::{HashSet, HashMap};
use std::hash::Hash;

pub struct DropoffWaveFront {
    last_score: usize,
    last_k: Option<i32>,
    wave_front_scores: Vec<WaveFrontScore>,
}
impl DropoffWaveFront {
    pub fn align_right_for_semi_global(
        ref_seq: Sequence,
        qry_seq: Sequence,
        penalties: &Penalties,
        spare_penalty: usize,
        position_of_checkpoints: HashMap<usize, PositionOfCheckpoint>,
    ) {
        let dropoff_wave_front = Self::new_with_align(ref_seq, qry_seq, penalties, spare_penalty, &consecutive_match_forward);

        if dropoff_wave_front.is_extended_to_end() {
            let owned_operations = dropoff_wave_front.backtrace_from_last_k(); //TODO: NEXT
        }
    }
    fn new_with_align(
        ref_seq: Sequence,
        qry_seq: Sequence,
        penalties: &Penalties,
        spare_penalty: usize,
        match_counter: MatchCounter,
    ) -> Self {
        let ref_len = ref_seq.len();
        let qry_len = qry_seq.len();

        let mut dropoff_wave_front = Self::allocated_empty(penalties, spare_penalty);

        let first_match_count = match_counter(ref_seq, qry_seq, 0, 0);

        dropoff_wave_front.wave_front_scores[0].add_first_components(first_match_count);
        
        if first_match_count as usize >= ref_len || first_match_count as usize >= qry_len {
            dropoff_wave_front.update_if_aligned_to_end(0);
            return dropoff_wave_front;
        }

        for score in 1..=spare_penalty {
            let optional_last_k = dropoff_wave_front.fill_wave_front_score_and_exist_with_last_k(ref_seq, qry_seq, ref_len, qry_len, score, penalties, match_counter);

            if let Some(last_k) = optional_last_k {
                dropoff_wave_front.update_if_aligned_to_end(last_k);
                return dropoff_wave_front;
            }
        }

        dropoff_wave_front
    }
    fn allocated_empty(penalties: &Penalties, spare_penalty: usize) -> Self {
        let wave_front_score_count = spare_penalty + 1;
        let gap_open_penalty = penalties.o;
        let gap_extend_penalty = penalties.e;

        let mut wave_front_scores: Vec<WaveFrontScore> = Vec::with_capacity(wave_front_score_count);

        let first_wave_front_score = WaveFrontScore::with_max_k(0);
        (0..gap_open_penalty + gap_extend_penalty).for_each(|_| {
            wave_front_scores.push(first_wave_front_score.clone());
        });

        if spare_penalty >= gap_open_penalty + gap_extend_penalty {
            let quot = ((spare_penalty - gap_open_penalty - gap_extend_penalty) / gap_extend_penalty) as i32;
            let rem = (spare_penalty - gap_open_penalty - gap_extend_penalty) % gap_extend_penalty;
            for max_k in 1..quot+1 {
                (0..gap_extend_penalty).for_each(|_| {
                    wave_front_scores.push(WaveFrontScore::with_max_k(max_k));
                });
            };
            (0..rem+1).for_each(|_| {
                wave_front_scores.push(WaveFrontScore::with_max_k(quot+1));
            });
        }

        Self {
            last_score: spare_penalty,
            last_k: None,
            wave_front_scores,
        }
    }
    fn fill_wave_front_score_and_exist_with_last_k(
        &mut self,
        ref_seq: Sequence,
        qry_seq: Sequence,
        ref_len: usize,
        qry_len: usize,
        score: usize,
        penalties: &Penalties,
        match_counter: MatchCounter,
    ) -> Option<i32> {
        let (mut components_of_score, range_of_k) = self.new_components_and_k_range_of_score(score, penalties);

        let wave_front_score = &mut self.wave_front_scores[score];

        for ([m_component, _, _], k) in components_of_score.iter_mut().zip(range_of_k.into_iter()) {
            if m_component.bt != EMPTY {
                // Extend & update
                let mut v = (m_component.fr - k) as usize;
                let mut h = m_component.fr as usize;
                let match_count = match_counter(ref_seq, qry_seq, v, h);
                m_component.fr += match_count;
                // Check exit condition
                v += match_count as usize;
                h += match_count as usize;
                if h >= ref_len || v >= qry_len {
                    wave_front_score.update(components_of_score);
                    return Some(k);
                }
            };
        };
        wave_front_score.update(components_of_score);
        None
    }
    fn new_components_and_k_range_of_score(&self, score: usize, penalties: &Penalties) -> (Components, Vec<i32>) { // TODO: Use const to indexing component
        let wave_front_score = &self.wave_front_scores[score];
        let mismatch_penalty = penalties.x;
        let gap_open_penalty = penalties.o;
        let gap_extend_penalty = penalties.e;

        let range_of_k = wave_front_score.range_of_k();

        let mut components: Components = vec![[Component::empty(); 3]; range_of_k.len()];
    
        // (1) From score: s-o-e
        if let Some(pre_score) = score.checked_sub(gap_open_penalty + gap_extend_penalty) {
            let max_k_of_pre_score = self.wave_front_scores[pre_score].max_k;
            let pre_wave_front_score = &self.wave_front_scores[pre_score];
            for (index_of_k, k) in range_of_k.iter().enumerate() {
                let component_of_k = &mut components[index_of_k];
                // 1. Update I from M & M from I
                let mut component_index = max_k_of_pre_score + k - 1;
                if let Some([pre_m, _, _]) = pre_wave_front_score.components.get(component_index as usize) {
                    if pre_m.bt != EMPTY {
                        // Update I
                        component_of_k[1] = Component {
                            fr: pre_m.fr + 1,
                            bt: FROM_M,
                        };
                        
                    }
                }
                // 2. Update D from M & M from D
                component_index += 2;
                if let Some([pre_m, _, _]) = pre_wave_front_score.components.get(component_index as usize) {
                    if pre_m.bt != EMPTY {
                        // Update D
                        component_of_k[2] = Component {
                            fr: pre_m.fr,
                            bt: FROM_M,
                        };
                    }
                }
            }
        }
        // (2) From score: s-e
        if let Some(pre_score) = score.checked_sub(gap_extend_penalty) {
            let pre_wave_front_score = &self.wave_front_scores[pre_score];
            range_of_k.iter().enumerate().for_each(|(index_of_k, k)| {
                let component_of_k = &mut components[index_of_k];
                // 1. Update I from I
                let mut component_index = pre_wave_front_score.max_k + k - 1;
                if let Some([_, pre_i, _]) = pre_wave_front_score.components.get(component_index as usize) {
                    if pre_i.bt != EMPTY {
                        // Update I
                        if component_of_k[1].bt == EMPTY || component_of_k[1].fr > pre_i.fr + 1 {
                            component_of_k[1] = Component {
                                fr: pre_i.fr + 1,
                                bt: FROM_I,
                            };
                        };
                    }
                }
                // 2. Update D from D
                component_index += 2;
                if let Some([_, _, pre_d]) = pre_wave_front_score.components.get(component_index as usize) {
                    if pre_d.bt != EMPTY {
                        // Update D
                        if component_of_k[2].bt == EMPTY || component_of_k[2].fr > pre_d.fr {
                            component_of_k[2] = Component {
                                fr: pre_d.fr,
                                bt: FROM_D,
                            };
                        };
                    }
                }
            });
        }
        // (3) From score: s-x
        if let Some(pre_score) = score.checked_sub(mismatch_penalty) {
            let pre_wave_front_score = &self.wave_front_scores[pre_score];
            range_of_k.iter().enumerate().for_each(|(index_of_k, k)| {
                let component_of_k = &mut components[index_of_k];
                // 1. Update M from M
                let component_index = pre_wave_front_score.max_k + k;
                if let Some([pre_m, _, _]) = pre_wave_front_score.components.get(component_index as usize) {
                    // Update M
                    component_of_k[0] = Component {
                        fr: pre_m.fr + 1,
                        bt: FROM_M,
                    };
                }
                // 2. Update M from I
                if component_of_k[1].bt != EMPTY {
                    if component_of_k[0].bt == EMPTY || component_of_k[1].fr >= component_of_k[0].fr {
                        component_of_k[0] = Component {
                            fr: component_of_k[1].fr,
                            bt: FROM_I,
                        };
                    };
                }
                // 3. Update M from D
                if component_of_k[2].bt != EMPTY {
                    if component_of_k[0].bt == EMPTY || component_of_k[2].fr >= component_of_k[0].fr {
                        component_of_k[0] = Component {
                            fr: component_of_k[2].fr,
                            bt: FROM_D,
                        };
                    };
                }
            });
        }

        (components, range_of_k)
    }
    fn update_if_aligned_to_end(&mut self, last_k: i32) {
        let last_score = self.wave_front_scores.len() + 1;
        self.wave_front_scores.truncate(last_score);
        self.last_score = last_score;
        self.last_k = Some(last_k);
    }
    fn is_extended_to_end(&self) -> bool {
        match self.last_k {
            Some(_) => true,
            None => false,
        }
    }
    fn backtrace_from_last_k(&self) {

    }
    fn backtrace_from_point(
        &self,
        mut score: usize,
        mut k: i32,
        penalties: &Penalties,
        current_anchor_index: usize,
        position_of_checkpoints: HashMap<usize, PositionOfCheckpoint>,
    ) {
        let wave_front_scores = &self.wave_front_scores;
        let mut operation_length: usize = 0;
        let mut operations: Vec<AlignmentOperation> = Vec::new(); // TODO: Capacity can be applied?
        let mut backtrace_extension_of_checkpoints: HashMap<usize, Extension> = HashMap::with_capacity(position_of_checkpoints.len());
        
        let mut wave_front_score: &WaveFrontScore = &wave_front_scores[score];
        let mut component_type: usize = M_COMPONENT;
        let mut component: &Component = wave_front_score.component_of_k(k, component_type);
        let mut fr: i32 = component.fr;
        
        loop {
            match component_type {
                /* M */
                M_COMPONENT => {
                    match component.bt {
                        FROM_M => {
                            // (1) Next score
                            score -= penalties.x;
                            // (2) Next k
                            // not change
                            // (3) Next WFS
                            wave_front_score = &wave_front_scores[score];
                            // (4) Component type
                            // not change
                            // (5) Next component
                            component = wave_front_score.component_of_k(k, M_COMPONENT);
                            // (6) Next fr
                            let next_fr = component.fr;
                            // (7) Add operation
                            let match_count = (fr - next_fr - 1) as u32;
                            if match_count == 0 {
                                if let Some(
                                    AlignmentOperation {
                                        alignment_type: AlignmentType::Subst,
                                        count: last_fr
                                    }) = operations.last_mut() {
                                    *last_fr += 1;
                                } else {
                                    operations.push(
                                        AlignmentOperation {
                                            alignment_type: AlignmentType::Subst,
                                            count: 1
                                        }
                                    );
                                }
                            } else {
                                operations.push(
                                    AlignmentOperation {
                                        alignment_type: AlignmentType::Match,
                                        count: match_count
                                    }
                                );
                                operations.push(
                                    AlignmentOperation {
                                        alignment_type: AlignmentType::Subst,
                                        count: 1
                                    }
                                );
                            }
                            operation_length += (match_count + 1) as usize;
                            // (8) Check if anchor is passed
                            for (&anchor_index, position_of_checkpoint) in &position_of_checkpoints {
                                if position_of_checkpoint.if_check_point_traversed(k, fr, next_fr) {
                                    let penalty = score + penalties.x;
                                    let length = operation_length - (position_of_checkpoint.fr - next_fr) as usize;
                                    let ref_to_operations = RefToOperations {
                                        anchor_index: current_anchor_index,
                                        start_point_of_operations: StartPointOfOperations {
                                            operation_index: operations.len() - 2,
                                            operation_count: (position_of_checkpoint.fr - fr) as u32,
                                        },
                                    };

                                    let extension = Extension {
                                        penalty,
                                        length, 
                                        operations: OperationsOfExtension::Ref(ref_to_operations),
                                    };

                                    backtrace_extension_of_checkpoints.insert(anchor_index, extension);
                                    position_of_checkpoints.remove(&anchor_index);
                                }
                            }
                            // (9) Next fr to fr
                            fr = next_fr;
                        },
                        FROM_I => {
                            // (1) Next score
                            // not change
                            // (2) Next k
                            // not change
                            // (3) Next WFS
                            // not change
                            // (4) Component type
                            component_type = 1;
                            // (5) Next component
                            component = wave_front_score.component_of_k(k, I_COMPONENT);
                            // (6) Next fr
                            let next_fr = component.fr;
                            // (7) Add Cigar
                            let match_count = (fr-next_fr) as u32;
                            if match_count != 0 {
                                operations.push(
                                    AlignmentOperation {
                                        alignment_type: AlignmentType::Match,
                                        count: match_count
                                    }
                                );
                            }
                            operation_length += match_count as usize;
                            // (8) Check if anchor is passed
                            /*
                            for checkpoint_index in valid_checkpoints_index.clone() {
                                let &(anchor_index, size, checkpoint_k, checkpoint_fr) = &check_points_values[checkpoint_index];
                                if (checkpoint_k == k) && (checkpoint_fr <= fr) && (checkpoint_fr - size >= next_fr) {
                                    checkpoint_backtrace.insert(
                                        anchor_index,
                                        (
                                            score,
                                            operation_length - (checkpoint_fr - next_fr) as usize,
                                        ),
                                    );
                                    valid_checkpoints_index.remove(&checkpoint_index);
                                }
                            }
                             */
                            // (9) Next fr to fr
                            fr = next_fr;
                        },
                        FROM_D => {
                            // (1) Next score
                            // not change
                            // (2) Next k
                            // not change
                            // (3) Next WFS
                            // not change
                            // (4) Component type
                            component_type = 2;
                            // (5) Next component
                            component = wave_front_score.component_of_k(k, D_COMPONENT);
                            // (6) Next fr
                            let next_fr = component.fr;
                            // (7) Add Cigar
                            let match_count = (fr-next_fr) as u32;
                            if match_count != 0 {
                                operations.push(
                                    AlignmentOperation {
                                        alignment_type: AlignmentType::Match,
                                        count: match_count
                                    }
                                );
                            }
                            operation_length += match_count as usize;
                            // (8) Check if anchor is passed
                            /*
                            for checkpoint_index in valid_checkpoints_index.clone() {
                                let &(anchor_index, size, checkpoint_k, checkpoint_fr) = &check_points_values[checkpoint_index];
                                if (checkpoint_k == k) && (checkpoint_fr <= fr) && (checkpoint_fr - size >= next_fr) {
                                    checkpoint_backtrace.insert(
                                        anchor_index,
                                        (
                                            score,
                                            operation_length - (checkpoint_fr - next_fr) as usize,
                                        ),
                                    );
                                    valid_checkpoints_index.remove(&checkpoint_index);
                                }
                            }
                             */
                            // (9) Next fr to fr
                            fr = next_fr;
                        },
                        _ => { // START_POINT
                            if fr != 0 {
                                operations.push(
                                    AlignmentOperation {
                                        alignment_type: AlignmentType::Match,
                                        count: fr as u32,
                                    }
                                );
                            };
                            operation_length += fr as usize;
                            // shrink
                            operations.shrink_to_fit();
                            backtrace_extension_of_checkpoints.shrink_to_fit();
                            // return ((cigar, operation_length), checkpoint_backtrace); //FIXME:
                        }
                    }
                },
                /* I */
                I_COMPONENT => {
                    match component.bt {
                        FROM_M => {
                            // (1) Next score
                            score -= penalties.o + penalties.e;
                            // (2) Next k
                            k -= 1;
                            // (3) Next WFS
                            wave_front_score = &wave_front_scores[score];
                            // (4) Component type
                            component_type = 0;
                            // (5) Next component
                            component = wave_front_score.component_of_k(k, M_COMPONENT);
                            // (6) Next fr
                            let next_fr = component.fr;
                            // (7) Add operation
                            if let Some(
                                AlignmentOperation {
                                    alignment_type: AlignmentType::Insertion,
                                    count: last_fr
                                }) = operations.last_mut() {
                                *last_fr += 1;
                            } else {
                                operations.push(
                                    AlignmentOperation {
                                        alignment_type: AlignmentType::Insertion,
                                        count: 1,
                                    }
                                )
                            }
                            operation_length += 1;
                            // (8) Check if anchor is passed
                            // not needed
                            // (9) Next fr to fr
                            fr = next_fr;
                        },
                        _ => { // FROM_I
                            // (1) Next score
                            score -= penalties.e;
                            // (2) Next k
                            k -= 1;
                            // (3) Next WFS
                            wave_front_score = &wave_front_scores[score];
                            // (4) Component type
                            // not change
                            // (5) Next component
                            component = wave_front_score.component_of_k(k, I_COMPONENT);
                            // (6) Next fr
                            let next_fr = component.fr;
                            // (7) Add operation
                            if let Some(
                                AlignmentOperation {
                                    alignment_type: AlignmentType::Insertion,
                                    count: last_fr
                                }) = operations.last_mut() {
                                *last_fr += 1;
                            } else {
                                operations.push(
                                    AlignmentOperation {
                                        alignment_type: AlignmentType::Insertion,
                                        count: 1,
                                    }
                                )
                            }
                            operation_length += 1;
                            // (8) Check if anchor is passed
                            // not needed
                            // (9) Next fr to fr
                            fr = next_fr;
                        },
                    }
                },
                /* D */
                _ => {
                    match component.bt {
                        FROM_M => {
                            // (1) Next score
                            score -= penalties.o + penalties.e;
                            // (2) Next k
                            k += 1;
                            // (3) Next WFS
                            wave_front_score = &wave_front_scores[score];
                            // (4) Component type
                            component_type = 0;
                            // (5) Next component
                            component = wave_front_score.component_of_k(k, M_COMPONENT);
                            // (6) Next fr
                            let next_fr = component.fr;
                            // (7) Add operation
                            if let Some(
                                AlignmentOperation {
                                    alignment_type: AlignmentType::Deletion,
                                    count: last_fr
                                }) = operations.last_mut() {
                                *last_fr += 1;
                            } else {
                                operations.push(
                                    AlignmentOperation {
                                        alignment_type: AlignmentType::Deletion,
                                        count: 1,
                                    }
                                )
                            }
                            operation_length += 1;
                            // (8) Check if anchor is passed
                            // not needed
                            // (9) Next fr to fr
                            fr = next_fr;
                        },
                        _ => { // FROM_D
                            // (1) Next score
                            score -= penalties.e;
                            // (2) Next k
                            k += 1;
                            // (3) Next WFS
                            wave_front_score = &wave_front_scores[score];
                            // (4) Component type
                            // not change
                            // (5) Next component
                            component = wave_front_score.component_of_k(k, D_COMPONENT);
                            // (6) Next fr
                            let next_fr = component.fr;
                            // (7) Add operation
                            if let Some(
                                AlignmentOperation {
                                    alignment_type: AlignmentType::Deletion,
                                    count: last_fr
                                }) = operations.last_mut() {
                                *last_fr += 1;
                            } else {
                                operations.push(
                                    AlignmentOperation {
                                        alignment_type: AlignmentType::Deletion,
                                        count: 1,
                                    }
                                )
                            }
                            operation_length += 1;
                            // (8) Check if anchor is passed
                            // not needed
                            // (9) Next fr to fr
                            fr = next_fr;
                        },
                    }
                },
            };
        }
    }
}

#[derive(Debug, Clone)]
struct WaveFrontScore {
    max_k: i32,
    components: Components,
}
impl WaveFrontScore {
    fn with_max_k(max_k: i32) -> Self {
        Self {
            max_k,
            components: Vec::new(),
        }
    }
    fn add_first_components(&mut self, first_match: i32) {
        self.components = vec![[
            Component { fr: first_match, bt: START },
            Component { fr: 0, bt: EMPTY } ,
            Component { fr: 0, bt: EMPTY } ,
        ]];
    }
    fn range_of_k(&self) -> Vec<i32> {
        (-self.max_k..=self.max_k).collect()
    }
    fn update(&mut self, new_components: Components) {
        self.components = new_components;
    }
    fn component_of_k(&self, k: i32, component_type: usize) -> &Component {
        &self.components[(self.max_k + k) as usize][component_type]
    }
}

// Component Index
const M_COMPONENT: usize = 0;
const I_COMPONENT: usize = 1;
const D_COMPONENT: usize = 2;

type Components = Vec<[Component; 3]>;

// Backtrace marker
const EMPTY: u8 = 0;
const FROM_M: u8 = 1;
const FROM_I: u8 = 2;
const FROM_D: u8 = 3;
const START: u8 = 4;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Component {
    fr: i32,
    bt: u8,
}

impl Component {
    fn empty() -> Self {
        Self { fr: 0 , bt: EMPTY }
    }
}

//TODO: Apply SIMD
fn consecutive_match_forward(ref_seq: &[u8], qry_seq: &[u8], v: usize, h: usize) -> i32 {
    let mut fr_to_add: i32 = 0;
    for (v1, v2) in qry_seq[v..].iter().zip(ref_seq[h..].iter()) {
        if *v1 == *v2 {
            fr_to_add += 1;
        } else {
            return fr_to_add
        }
    }
    fr_to_add
}
fn consecutive_match_reverse(ref_seq: &[u8], qry_seq: &[u8], v: usize, h: usize) -> i32 {
    let mut fr_to_add: i32 = 0;
    for (v1, v2) in qry_seq[..qry_seq.len()-v].iter().rev().zip(ref_seq[..ref_seq.len()-h].iter().rev()) {
        if *v1 == *v2 {
            fr_to_add += 1;
        } else {
            return fr_to_add
        }
    }
    fr_to_add
}

pub struct PositionOfCheckpoint {
    k: i32,
    fr: i32,
    anchor_size: i32,
}
impl PositionOfCheckpoint {
    pub fn new(ref_gap: usize, qry_gap: usize, anchor_size: usize) -> Self {
        Self {
            k: (ref_gap - qry_gap) as i32,
            fr: ref_gap as i32,
            anchor_size: anchor_size as i32,
        }
    }
    fn if_check_point_traversed(&self, k: i32, fr: i32, next_fr: i32) -> bool {
        (self.k == k) && (self.fr <= fr) && (self.fr - self.anchor_size >= next_fr)
    }
}
