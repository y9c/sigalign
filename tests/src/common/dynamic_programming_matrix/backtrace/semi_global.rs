use sigalign::results::Alignment;

use super::{
    DpMatrix,
    parse_the_unoverlapped_alignments_with_path,
};

pub fn parse_valid_semi_global_result_from_dpm(
    dp_matrix: &DpMatrix,
    minimum_length: u32,
    maximum_penalty_per_length: f32,
) -> Vec<Alignment> {
    let unoverlapped_alignments_with_path = parse_the_unoverlapped_alignments_with_path(
        dp_matrix,
        0,
        dp_matrix.query.len() - 1,
    );

    unoverlapped_alignments_with_path.into_iter().filter_map(|(x, _)| {
        let length = x.length;
        let penalty = x.penalty;
        if (
            length >= minimum_length
        ) && (
            penalty <= (length as f64 * maximum_penalty_per_length as f64) as u32
        ) {
            Some(x)
        } else {
            None
        }
    }).collect()
}