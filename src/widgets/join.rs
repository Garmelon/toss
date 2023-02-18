use std::cmp::Ordering;

// The following algorithm has three goals, listed in order of importance:
//
// 1. Use the available space
// 2. Avoid shrinking segments where possible
// 3. Match the given weights as closely as possible
//
// Its input is a list of weighted segments where each segment wants to use a
// certain amount of space. The weights signify how the available space would be
// assigned if goal 2 was irrelevant.
//
// First, the algorithm must decide whether it must grow or shrink segments.
// Because goal 2 has a higher priority than goal 3, it never makes sense to
// shrink a segment in order to make another larger. In both cases, a segment's
// actual size is compared to its allotment, i. e. what size it should be based
// on its weight.
//
// Growth
// ======
//
// If segments must be grown, an important observation can be made: If all
// segments are smaller than their allotment, then each segment can be assigned
// its allotment without violating goal 2, thereby fulfilling goal 3.
//
// Another important observation can be made: If a segment is at least as large
// as its allotment, it must never be grown as that would violate goal 3.
//
// Based on these two observations, the growth algorithm first repeatedly
// removes all segments that are at least as large as their allotment. It then
// resizes the remaining segments to their allotments.
//
// Shrinkage
// =========
//
// If segments must be shrunk, an important observation can be made: If all
// segments are larger than their allotment, then each segment can be assigned
// its allotment, thereby fulfilling goal 3. Since goal 1 is more important than
// goal 2, we know that some elements must be shrunk.
//
// Another important observation can be made: If a segment is at least as small
// as its allotment, it must never be shrunk as that would violate goal 3.
//
// Based on these two observations, the shrinkage algorithm first repeatedly
// removes all segments that are at least as small as their allotment. It then
// resizes the remaining segments to their allotments.

struct Segment {
    size: u16,
    weight: f32,
}

fn balance(segments: &mut [Segment], available: u16) {
    if segments.is_empty() {
        return;
    }

    let total_size = segments.iter().map(|s| s.size).sum::<u16>();
    match total_size.cmp(&available) {
        Ordering::Less => grow(segments, available),
        Ordering::Greater => shrink(segments, available),
        Ordering::Equal => {}
    }

    assert!(available >= segments.iter().map(|s| s.size).sum::<u16>());
}

fn grow(segments: &mut [Segment], mut available: u16) {
    assert!(available > segments.iter().map(|s| s.size).sum::<u16>());
    let mut segments = segments.iter_mut().collect::<Vec<_>>();

    // Repeatedly remove all segments that do not need to grow, i. e. that are
    // at least as large as their allotment.
    loop {
        let mut total_weight = segments.iter().map(|s| s.weight).sum::<f32>();

        // If there are no segments with a weight > 0, space is distributed
        // evenly among all remaining segments.
        if total_weight <= 0.0 {
            for segment in &mut segments {
                segment.weight = 1.0;
            }
            total_weight = segments.len() as f32;
        }

        let mut changed = false;
        segments.retain(|s| {
            let allotment = s.weight / total_weight * available as f32;
            if (s.size as f32) < allotment {
                return true; // May need to grow
            }
            available -= s.size;
            changed = true;
            false
        });

        // If all segments were at least as large as their allotments, we would
        // be trying to shrink, not grow them. Hence, there must be at least one
        // segment that is smaller than its allotment.
        assert!(!segments.is_empty());

        if !changed {
            break; // All remaining segments are smaller than their allotments
        }
    }

    // Size each remaining segment according to its allotment.
    let total_weight = segments.iter().map(|s| s.weight).sum::<f32>();
    let mut used = 0;
    for segment in &mut segments {
        let allotment = segment.weight / total_weight * available as f32;
        segment.size = allotment.floor() as u16;
        used += segment.size;
    }

    // Distribute remaining unused space from left to right.
    //
    // The rounding error on each segment is at most 1, so we only need to loop
    // over the segments once.
    let remaining = available - used;
    assert!(remaining as usize <= segments.len());
    for segment in segments.into_iter().take(remaining.into()) {
        segment.size += 1;
    }
}

fn shrink(segments: &mut [Segment], mut available: u16) {
    assert!(available < segments.iter().map(|s| s.size).sum::<u16>());
    let mut segments = segments.iter_mut().collect::<Vec<_>>();

    // Repeatedly remove all segments that do not need to shrink, i. e. that are
    // at least as small as their allotment.
    loop {
        let mut total_weight = segments.iter().map(|s| s.weight).sum::<f32>();

        // If there are no segments with a weight > 0, space is distributed
        // evenly among all remaining segments.
        if total_weight <= 0.0 {
            for segment in &mut segments {
                segment.weight = 1.0;
            }
            total_weight = segments.len() as f32;
        }

        let mut changed = false;
        segments.retain(|s| {
            let allotment = s.weight / total_weight * available as f32;
            if (s.size as f32) > allotment {
                return true; // May need to shrink
            }

            // The size subtracted from `available` is always smaller than or
            // equal to its allotment. It must be smaller in at least one case,
            // or we wouldn't be shrinking. Since `available` is the sum of all
            // allotments, it never reaches 0.
            assert!(available > s.size);

            available -= s.size;
            changed = true;
            false
        });

        // If all segments were smaller or the same size as their allotments, we
        // would be trying to grow, not shrink them. Hence, there must be at
        // least one segment bigger than its allotment.
        assert!(!segments.is_empty());

        if !changed {
            break; // All segments want more than their weight allows.
        }
    }

    // Size each remaining segment according to its allotment.
    let total_weight = segments.iter().map(|s| s.weight).sum::<f32>();
    let mut used = 0;
    for segment in &mut segments {
        let allotment = segment.weight / total_weight * available as f32;
        segment.size = allotment.floor() as u16;
        used += segment.size;
    }

    // Distribute remaining unused space from left to right.
    //
    // The rounding error on each segment is at most 1, so we only need to loop
    // over the segments once.
    let remaining = available - used;
    assert!(remaining as usize <= segments.len());
    for segment in segments.into_iter().take(remaining.into()) {
        segment.size += 1;
    }
}
