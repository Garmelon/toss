use std::cmp::Ordering;

use async_trait::async_trait;

use crate::{AsyncWidget, Frame, Pos, Size, Widget};

use super::{Either2, Either3, Either4, Either5, Either6, Either7};

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

#[derive(Debug)]
struct Segment {
    size: u16,
    weight: f32,
    fixed: bool,
}

impl Segment {
    fn horizontal<I>(size: Size, segment: &JoinSegment<I>) -> Self {
        Self {
            size: size.width,
            weight: segment.weight,
            fixed: segment.fixed,
        }
    }

    fn vertical<I>(size: Size, segment: &JoinSegment<I>) -> Self {
        Self {
            size: size.height,
            weight: segment.weight,
            fixed: segment.fixed,
        }
    }
}

fn total_size(segments: &[&mut Segment]) -> u16 {
    let mut total = 0_u16;
    for segment in segments {
        total = total.saturating_add(segment.size);
    }
    total
}

fn total_weight(segments: &[&mut Segment]) -> f32 {
    segments.iter().map(|s| s.weight).sum()
}

fn balance(segments: &mut [Segment], mut available: u16) {
    let mut borrowed_segments = segments.iter_mut().collect::<Vec<_>>();

    // Remove fixed segments
    borrowed_segments.retain(|s| {
        if !s.fixed {
            return true;
        }
        available = available.saturating_sub(s.size);
        false
    });

    if borrowed_segments.is_empty() || available == 0 {
        return;
    }

    match total_size(&borrowed_segments).cmp(&available) {
        Ordering::Less => grow(borrowed_segments, available),
        Ordering::Greater => shrink(borrowed_segments, available),
        Ordering::Equal => {}
    }
}

fn grow(mut segments: Vec<&mut Segment>, mut available: u16) {
    assert!(available > total_size(&segments));

    // Repeatedly remove all segments that do not need to grow, i. e. that are
    // at least as large as their allotment.
    loop {
        let mut total_weight = total_weight(&segments);

        // If there are no segments with a weight > 0, space is distributed
        // evenly among all remaining segments.
        if total_weight <= 0.0 {
            for segment in &mut segments {
                segment.weight = 1.0;
            }
            total_weight = segments.len() as f32;
        }

        let mut removed = 0;
        segments.retain(|s| {
            let allotment = s.weight / total_weight * available as f32;
            if (s.size as f32) < allotment {
                return true; // May need to grow
            }
            removed += s.size;
            false
        });
        available -= removed;

        // If all segments were at least as large as their allotments, we would
        // be trying to shrink, not grow them. Hence, there must be at least one
        // segment that is smaller than its allotment.
        assert!(!segments.is_empty());

        if removed == 0 {
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

fn shrink(mut segments: Vec<&mut Segment>, mut available: u16) {
    assert!(available < total_size(&segments));

    // Repeatedly remove all segments that do not need to shrink, i. e. that are
    // at least as small as their allotment.
    loop {
        let mut total_weight = total_weight(&segments);

        // If there are no segments with a weight > 0, space is distributed
        // evenly among all remaining segments.
        if total_weight <= 0.0 {
            for segment in &mut segments {
                segment.weight = 1.0;
            }
            total_weight = segments.len() as f32;
        }

        let mut removed = 0;
        segments.retain(|s| {
            let allotment = s.weight / total_weight * available as f32;
            if (s.size as f32) > allotment {
                return true; // May need to shrink
            }

            // The segment size subtracted from `available` is always smaller
            // than or equal to its allotment. Since `available` is the sum of
            // all allotments, it can never go below 0.
            assert!(s.size <= available);

            removed += s.size;
            false
        });
        available -= removed;

        // If all segments were smaller or the same size as their allotments, we
        // would be trying to grow, not shrink them. Hence, there must be at
        // least one segment bigger than its allotment.
        assert!(!segments.is_empty());

        if removed == 0 {
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

pub struct JoinSegment<I> {
    inner: I,
    weight: f32,
    fixed: bool,
}

impl<I> JoinSegment<I> {
    pub fn new(inner: I) -> Self {
        Self {
            inner,
            weight: 1.0,
            fixed: false,
        }
    }

    pub fn weight(mut self, weight: f32) -> Self {
        assert!(weight >= 0.0);
        self.weight = weight;
        self
    }

    pub fn fixed(mut self, fixed: bool) -> Self {
        self.fixed = fixed;
        self
    }
}

pub struct JoinH<I> {
    segments: Vec<JoinSegment<I>>,
}

impl<I> JoinH<I> {
    pub fn new(segments: Vec<JoinSegment<I>>) -> Self {
        Self { segments }
    }
}

impl<E, I> Widget<E> for JoinH<I>
where
    I: Widget<E>,
{
    fn size(
        &self,
        frame: &mut Frame,
        max_width: Option<u16>,
        max_height: Option<u16>,
    ) -> Result<Size, E> {
        if let Some(max_width) = max_width {
            let mut balanced_segments = vec![];
            for segment in &self.segments {
                let size = segment.inner.size(frame, Some(max_width), max_height)?;
                balanced_segments.push(Segment::horizontal(size, segment));
            }
            balance(&mut balanced_segments, max_width);

            let mut width = 0_u16;
            let mut height = 0_u16;
            for (segment, balanced) in self.segments.iter().zip(balanced_segments.into_iter()) {
                let size = segment.inner.size(frame, Some(balanced.size), max_height)?;
                width = width.saturating_add(size.width);
                height = height.max(size.height);
            }
            Ok(Size::new(width, height))
        } else {
            let mut width = 0_u16;
            let mut height = 0_u16;
            for segment in &self.segments {
                let size = segment.inner.size(frame, max_width, max_height)?;
                width = width.saturating_add(size.width);
                height = height.max(size.height);
            }
            Ok(Size::new(width, height))
        }
    }

    fn draw(self, frame: &mut Frame) -> Result<(), E> {
        let size = frame.size();
        let max_width = Some(size.width);
        let max_height = Some(size.height);

        let mut balanced_segments = vec![];
        for segment in &self.segments {
            let size = segment.inner.size(frame, max_width, max_height)?;
            balanced_segments.push(Segment::horizontal(size, segment));
        }
        balance(&mut balanced_segments, size.width);

        let mut x = 0;
        for (segment, balanced) in self.segments.into_iter().zip(balanced_segments.into_iter()) {
            frame.push(Pos::new(x, 0), Size::new(balanced.size, size.height));
            segment.inner.draw(frame)?;
            frame.pop();
            x += balanced.size as i32;
        }

        Ok(())
    }
}

#[async_trait]
impl<E, I> AsyncWidget<E> for JoinH<I>
where
    I: AsyncWidget<E> + Send + Sync,
{
    async fn size(
        &self,
        frame: &mut Frame,
        max_width: Option<u16>,
        max_height: Option<u16>,
    ) -> Result<Size, E> {
        if let Some(max_width) = max_width {
            let mut balanced_segments = vec![];
            for segment in &self.segments {
                let size = segment
                    .inner
                    .size(frame, Some(max_width), max_height)
                    .await?;
                balanced_segments.push(Segment::horizontal(size, segment));
            }
            balance(&mut balanced_segments, max_width);

            let mut width = 0_u16;
            let mut height = 0_u16;
            for (segment, balanced) in self.segments.iter().zip(balanced_segments.into_iter()) {
                let size = segment
                    .inner
                    .size(frame, Some(balanced.size), max_height)
                    .await?;
                width = width.saturating_add(size.width);
                height = height.max(size.height);
            }
            Ok(Size::new(width, height))
        } else {
            let mut width = 0_u16;
            let mut height = 0_u16;
            for segment in &self.segments {
                let size = segment.inner.size(frame, max_width, max_height).await?;
                width = width.saturating_add(size.width);
                height = height.max(size.height);
            }
            Ok(Size::new(width, height))
        }
    }

    async fn draw(self, frame: &mut Frame) -> Result<(), E> {
        let size = frame.size();
        let max_width = Some(size.width);
        let max_height = Some(size.height);

        let mut balanced_segments = vec![];
        for segment in &self.segments {
            let size = segment.inner.size(frame, max_width, max_height).await?;
            balanced_segments.push(Segment::horizontal(size, segment));
        }
        balance(&mut balanced_segments, size.width);

        let mut x = 0;
        for (segment, balanced) in self.segments.into_iter().zip(balanced_segments.into_iter()) {
            frame.push(Pos::new(x, 0), Size::new(balanced.size, size.height));
            segment.inner.draw(frame).await?;
            frame.pop();
            x += balanced.size as i32;
        }

        Ok(())
    }
}

pub struct JoinV<I> {
    segments: Vec<JoinSegment<I>>,
}

impl<I> JoinV<I> {
    pub fn new(segments: Vec<JoinSegment<I>>) -> Self {
        Self { segments }
    }
}

impl<E, I> Widget<E> for JoinV<I>
where
    I: Widget<E>,
{
    fn size(
        &self,
        frame: &mut Frame,
        max_width: Option<u16>,
        max_height: Option<u16>,
    ) -> Result<Size, E> {
        if let Some(max_height) = max_height {
            let mut balanced_segments = vec![];
            for segment in &self.segments {
                let size = segment.inner.size(frame, max_width, Some(max_height))?;
                balanced_segments.push(Segment::vertical(size, segment));
            }
            balance(&mut balanced_segments, max_height);

            let mut width = 0_u16;
            let mut height = 0_u16;
            for (segment, balanced) in self.segments.iter().zip(balanced_segments.into_iter()) {
                let size = segment.inner.size(frame, max_width, Some(balanced.size))?;
                width = width.max(size.width);
                height = height.saturating_add(size.height);
            }
            Ok(Size::new(width, height))
        } else {
            let mut width = 0_u16;
            let mut height = 0_u16;
            for segment in &self.segments {
                let size = segment.inner.size(frame, max_width, max_height)?;
                width = width.max(size.width);
                height = height.saturating_add(size.height);
            }
            Ok(Size::new(width, height))
        }
    }

    fn draw(self, frame: &mut Frame) -> Result<(), E> {
        let size = frame.size();
        let max_width = Some(size.width);
        let max_height = Some(size.height);

        let mut balanced_segments = vec![];
        for segment in &self.segments {
            let size = segment.inner.size(frame, max_width, max_height)?;
            balanced_segments.push(Segment::vertical(size, segment));
        }
        balance(&mut balanced_segments, size.height);

        let mut y = 0;
        for (segment, balanced) in self.segments.into_iter().zip(balanced_segments.into_iter()) {
            frame.push(Pos::new(0, y), Size::new(size.width, balanced.size));
            segment.inner.draw(frame)?;
            frame.pop();
            y += balanced.size as i32;
        }

        Ok(())
    }
}

#[async_trait]
impl<E, I> AsyncWidget<E> for JoinV<I>
where
    I: AsyncWidget<E> + Send + Sync,
{
    async fn size(
        &self,
        frame: &mut Frame,
        max_width: Option<u16>,
        max_height: Option<u16>,
    ) -> Result<Size, E> {
        if let Some(max_height) = max_height {
            let mut balanced_segments = vec![];
            for segment in &self.segments {
                let size = segment
                    .inner
                    .size(frame, max_width, Some(max_height))
                    .await?;
                balanced_segments.push(Segment::vertical(size, segment));
            }
            balance(&mut balanced_segments, max_height);

            let mut width = 0_u16;
            let mut height = 0_u16;
            for (segment, balanced) in self.segments.iter().zip(balanced_segments.into_iter()) {
                let size = segment
                    .inner
                    .size(frame, max_width, Some(balanced.size))
                    .await?;
                width = width.max(size.width);
                height = height.saturating_add(size.height);
            }
            Ok(Size::new(width, height))
        } else {
            let mut width = 0_u16;
            let mut height = 0_u16;
            for segment in &self.segments {
                let size = segment.inner.size(frame, max_width, max_height).await?;
                width = width.max(size.width);
                height = height.saturating_add(size.height);
            }
            Ok(Size::new(width, height))
        }
    }

    async fn draw(self, frame: &mut Frame) -> Result<(), E> {
        let size = frame.size();
        let max_width = Some(size.width);
        let max_height = Some(size.height);

        let mut balanced_segments = vec![];
        for segment in &self.segments {
            let size = segment.inner.size(frame, max_width, max_height).await?;
            balanced_segments.push(Segment::vertical(size, segment));
        }
        balance(&mut balanced_segments, size.height);

        let mut y = 0;
        for (segment, balanced) in self.segments.into_iter().zip(balanced_segments.into_iter()) {
            frame.push(Pos::new(0, y), Size::new(size.width, balanced.size));
            segment.inner.draw(frame).await?;
            frame.pop();
            y += balanced.size as i32;
        }

        Ok(())
    }
}

macro_rules! mk_join {
    (
        $name:ident: $base:ident + $either:ident {
            $( $arg:ident: $constr:ident ($ty:ident), )+
        }
    ) => {
        pub struct $name< $( $ty ),+ >($base<$either< $( $ty ),+ >>);

        impl< $( $ty ),+ > $name< $( $ty ),+ > {
            pub fn new( $( $arg: JoinSegment<$ty> ),+ ) -> Self {
                Self($base::new(vec![ $(
                    JoinSegment {
                        inner: $either::$constr($arg.inner),
                        weight: $arg.weight,
                        fixed: $arg.fixed,
                    },
                )+ ]))
            }
        }

        impl<E, $( $ty ),+ > Widget<E> for $name< $( $ty ),+ >
        where
            $( $ty: Widget<E>, )+
        {
            fn size(
                &self,
                frame: &mut Frame,
                max_width: Option<u16>,
                max_height: Option<u16>,
            ) -> Result<Size, E> {
                self.0.size(frame, max_width, max_height)
            }

            fn draw(self, frame: &mut Frame) -> Result<(), E> {
                self.0.draw(frame)
            }
        }

        #[async_trait]
        impl<E, $( $ty ),+ > AsyncWidget<E> for $name< $( $ty ),+ >
        where
            $( $ty: AsyncWidget<E> + Send + Sync, )+
        {
            async fn size(
                &self,
                frame: &mut Frame,
                max_width: Option<u16>,
                max_height: Option<u16>,
            ) -> Result<Size, E> {
                self.0.size(frame, max_width, max_height).await
            }

            async fn draw(self, frame: &mut Frame) -> Result<(), E> {
                self.0.draw(frame).await
            }
        }
    };
}

mk_join! {
    JoinH2: JoinH + Either2 {
        first: First(I1),
        second: Second(I2),
    }
}

mk_join! {
    JoinH3: JoinH + Either3 {
        first: First(I1),
        second: Second(I2),
        third: Third(I3),
    }
}

mk_join! {
    JoinH4: JoinH + Either4 {
        first: First(I1),
        second: Second(I2),
        third: Third(I3),
        fourth: Fourth(I4),
    }
}

mk_join! {
    JoinH5: JoinH + Either5 {
        first: First(I1),
        second: Second(I2),
        third: Third(I3),
        fourth: Fourth(I4),
        fifth: Fifth(I5),
    }
}

mk_join! {
    JoinH6: JoinH + Either6 {
        first: First(I1),
        second: Second(I2),
        third: Third(I3),
        fourth: Fourth(I4),
        fifth: Fifth(I5),
        sixth: Sixth(I6),
    }
}

mk_join! {
    JoinH7: JoinH + Either7 {
        first: First(I1),
        second: Second(I2),
        third: Third(I3),
        fourth: Fourth(I4),
        fifth: Fifth(I5),
        sixth: Sixth(I6),
        seventh: Seventh(I7),
    }
}

mk_join! {
    JoinV2: JoinV + Either2 {
        first: First(I1),
        second: Second(I2),
    }
}

mk_join! {
    JoinV3: JoinV + Either3 {
        first: First(I1),
        second: Second(I2),
        third: Third(I3),
    }
}

mk_join! {
    JoinV4: JoinV + Either4 {
        first: First(I1),
        second: Second(I2),
        third: Third(I3),
        fourth: Fourth(I4),
    }
}

mk_join! {
    JoinV5: JoinV + Either5 {
        first: First(I1),
        second: Second(I2),
        third: Third(I3),
        fourth: Fourth(I4),
        fifth: Fifth(I5),
    }
}

mk_join! {
    JoinV6: JoinV + Either6 {
        first: First(I1),
        second: Second(I2),
        third: Third(I3),
        fourth: Fourth(I4),
        fifth: Fifth(I5),
        sixth: Sixth(I6),
    }
}

mk_join! {
    JoinV7: JoinV + Either7 {
        first: First(I1),
        second: Second(I2),
        third: Third(I3),
        fourth: Fourth(I4),
        fifth: Fifth(I5),
        sixth: Sixth(I6),
        seventh: Seventh(I7),
    }
}
