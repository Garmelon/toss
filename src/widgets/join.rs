use std::cmp::Ordering;

use async_trait::async_trait;

use crate::{AsyncWidget, Frame, Pos, Size, Widget, WidthDb};

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
    major: u16,
    minor: u16,
    weight: f32,
    growing: bool,
    shrinking: bool,
}

impl Segment {
    fn new<I>(major_minor: (u16, u16), segment: &JoinSegment<I>) -> Self {
        Self {
            major: major_minor.0,
            minor: major_minor.1,
            weight: segment.weight,
            growing: segment.growing,
            shrinking: segment.shrinking,
        }
    }
}

fn total_size(segments: &[&mut Segment]) -> u16 {
    let mut total = 0_u16;
    for segment in segments {
        total = total.saturating_add(segment.major);
    }
    total
}

fn total_weight(segments: &[&mut Segment]) -> f32 {
    segments.iter().map(|s| s.weight).sum()
}

fn balance(segments: &mut [Segment], available: u16) {
    let segments = segments.iter_mut().collect::<Vec<_>>();
    match total_size(&segments).cmp(&available) {
        Ordering::Less => grow(segments, available),
        Ordering::Greater => shrink(segments, available),
        Ordering::Equal => {}
    }
}

fn grow(mut segments: Vec<&mut Segment>, mut available: u16) {
    assert!(available >= total_size(&segments));

    // Only grow segments that can be grown.
    segments.retain(|s| {
        if s.growing {
            return true;
        }
        available = available.saturating_sub(s.major);
        false
    });

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
            if (s.major as f32) < allotment {
                return true; // May need to grow
            }
            removed += s.major;
            false
        });
        available -= removed;

        if removed == 0 {
            break; // All remaining segments are smaller than their allotments
        }
    }

    let total_weight = segments.iter().map(|s| s.weight).sum::<f32>();
    if total_weight <= 0.0 {
        return; // No more segments left
    }

    // Size each remaining segment according to its allotment.
    let mut used = 0;
    for segment in &mut segments {
        let allotment = segment.weight / total_weight * available as f32;
        segment.major = allotment.floor() as u16;
        used += segment.major;
    }

    // Distribute remaining unused space from left to right.
    //
    // The rounding error on each segment is at most 1, so we only need to loop
    // over the segments once.
    let remaining = available - used;
    assert!(remaining as usize <= segments.len());
    for segment in segments.into_iter().take(remaining.into()) {
        segment.major += 1;
    }
}

fn shrink(mut segments: Vec<&mut Segment>, mut available: u16) {
    assert!(available <= total_size(&segments));

    // Only shrink segments that can be shrunk.
    segments.retain(|s| {
        if s.shrinking {
            return true;
        }
        available = available.saturating_sub(s.major);
        false
    });

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
            if (s.major as f32) > allotment {
                return true; // May need to shrink
            }

            // The segment size subtracted from `available` is always smaller
            // than or equal to its allotment. Since `available` is the sum of
            // all allotments, it can never go below 0.
            assert!(s.major <= available);

            removed += s.major;
            false
        });
        available -= removed;

        if removed == 0 {
            break; // All segments want more than their weight allows.
        }
    }

    let total_weight = segments.iter().map(|s| s.weight).sum::<f32>();
    if total_weight <= 0.0 {
        return; // No more segments left
    }

    // Size each remaining segment according to its allotment.
    let mut used = 0;
    for segment in &mut segments {
        let allotment = segment.weight / total_weight * available as f32;
        segment.major = allotment.floor() as u16;
        used += segment.major;
    }

    // Distribute remaining unused space from left to right.
    //
    // The rounding error on each segment is at most 1, so we only need to loop
    // over the segments once.
    let remaining = available - used;
    assert!(remaining as usize <= segments.len());
    for segment in segments.into_iter().take(remaining.into()) {
        segment.major += 1;
    }
}

pub struct JoinSegment<I> {
    pub inner: I,
    weight: f32,
    pub growing: bool,
    pub shrinking: bool,
}

impl<I> JoinSegment<I> {
    pub fn new(inner: I) -> Self {
        Self {
            inner,
            weight: 1.0,
            growing: true,
            shrinking: true,
        }
    }

    pub fn weight(&self) -> f32 {
        self.weight
    }

    pub fn set_weight(&mut self, weight: f32) {
        assert!(weight >= 0.0);
        self.weight = weight;
    }

    pub fn with_weight(mut self, weight: f32) -> Self {
        self.set_weight(weight);
        self
    }

    pub fn with_growing(mut self, enabled: bool) -> Self {
        self.growing = enabled;
        self
    }

    pub fn with_shrinking(mut self, enabled: bool) -> Self {
        self.shrinking = enabled;
        self
    }

    pub fn with_fixed(self, fixed: bool) -> Self {
        self.with_growing(!fixed).with_shrinking(!fixed)
    }
}

fn to_mm<T>(horizontal: bool, w: T, h: T) -> (T, T) {
    if horizontal {
        (w, h)
    } else {
        (h, w)
    }
}

fn from_mm<T>(horizontal: bool, major: T, minor: T) -> (T, T) {
    if horizontal {
        (major, minor)
    } else {
        (minor, major)
    }
}

fn size<E, I: Widget<E>>(
    horizontal: bool,
    widthdb: &mut WidthDb,
    segment: &JoinSegment<I>,
    major: Option<u16>,
    minor: Option<u16>,
) -> Result<(u16, u16), E> {
    if horizontal {
        let size = segment.inner.size(widthdb, major, minor)?;
        Ok((size.width, size.height))
    } else {
        let size = segment.inner.size(widthdb, minor, major)?;
        Ok((size.height, size.width))
    }
}

fn size_with_balanced<E, I: Widget<E>>(
    horizontal: bool,
    widthdb: &mut WidthDb,
    segment: &JoinSegment<I>,
    balanced: &Segment,
    minor: Option<u16>,
) -> Result<(u16, u16), E> {
    size(horizontal, widthdb, segment, Some(balanced.major), minor)
}

async fn size_async<E, I: AsyncWidget<E>>(
    horizontal: bool,
    widthdb: &mut WidthDb,
    segment: &JoinSegment<I>,
    major: Option<u16>,
    minor: Option<u16>,
) -> Result<(u16, u16), E> {
    if horizontal {
        let size = segment.inner.size(widthdb, major, minor).await?;
        Ok((size.width, size.height))
    } else {
        let size = segment.inner.size(widthdb, minor, major).await?;
        Ok((size.height, size.width))
    }
}

async fn size_async_with_balanced<E, I: AsyncWidget<E>>(
    horizontal: bool,
    widthdb: &mut WidthDb,
    segment: &JoinSegment<I>,
    balanced: &Segment,
    minor: Option<u16>,
) -> Result<(u16, u16), E> {
    size_async(horizontal, widthdb, segment, Some(balanced.major), minor).await
}

fn sum_major_max_minor(segments: &[Segment]) -> (u16, u16) {
    let mut major = 0_u16;
    let mut minor = 0_u16;
    for segment in segments {
        major = major.saturating_add(segment.major);
        minor = minor.max(segment.minor);
    }
    (major, minor)
}

pub struct Join<I> {
    horizontal: bool,
    segments: Vec<JoinSegment<I>>,
}

impl<I> Join<I> {
    pub fn horizontal(segments: Vec<JoinSegment<I>>) -> Self {
        Self {
            horizontal: true,
            segments,
        }
    }

    pub fn vertical(segments: Vec<JoinSegment<I>>) -> Self {
        Self {
            horizontal: false,
            segments,
        }
    }
}

impl<E, I> Widget<E> for Join<I>
where
    I: Widget<E>,
{
    fn size(
        &self,
        widthdb: &mut WidthDb,
        max_width: Option<u16>,
        max_height: Option<u16>,
    ) -> Result<Size, E> {
        let (max_major, max_minor) = to_mm(self.horizontal, max_width, max_height);

        let mut segments = Vec::with_capacity(self.segments.len());
        for segment in &self.segments {
            let major_minor = size(self.horizontal, widthdb, segment, None, max_minor)?;
            segments.push(Segment::new(major_minor, segment));
        }

        if let Some(available) = max_major {
            balance(&mut segments, available);

            let mut new_segments = Vec::with_capacity(self.segments.len());
            for (segment, balanced) in self.segments.iter().zip(segments.into_iter()) {
                let major_minor =
                    size_with_balanced(self.horizontal, widthdb, segment, &balanced, max_minor)?;
                new_segments.push(Segment::new(major_minor, segment));
            }
            segments = new_segments;
        }

        let (major, minor) = sum_major_max_minor(&segments);
        let (width, height) = from_mm(self.horizontal, major, minor);
        Ok(Size::new(width, height))
    }

    fn draw(self, frame: &mut Frame) -> Result<(), E> {
        let frame_size = frame.size();
        let (max_major, max_minor) = to_mm(self.horizontal, frame_size.width, frame_size.height);

        let widthdb = frame.widthdb();
        let mut segments = Vec::with_capacity(self.segments.len());
        for segment in &self.segments {
            let major_minor = size(self.horizontal, widthdb, segment, None, Some(max_minor))?;
            segments.push(Segment::new(major_minor, segment));
        }
        balance(&mut segments, max_major);

        let mut major = 0_i32;
        for (segment, balanced) in self.segments.into_iter().zip(segments.into_iter()) {
            let (x, y) = from_mm(self.horizontal, major, 0);
            let (w, h) = from_mm(self.horizontal, balanced.major, max_minor);
            frame.push(Pos::new(x, y), Size::new(w, h));
            segment.inner.draw(frame)?;
            frame.pop();
            major += balanced.major as i32;
        }

        Ok(())
    }
}

#[async_trait]
impl<E, I> AsyncWidget<E> for Join<I>
where
    I: AsyncWidget<E> + Send + Sync,
{
    async fn size(
        &self,
        widthdb: &mut WidthDb,
        max_width: Option<u16>,
        max_height: Option<u16>,
    ) -> Result<Size, E> {
        let (max_major, max_minor) = to_mm(self.horizontal, max_width, max_height);

        let mut segments = Vec::with_capacity(self.segments.len());
        for segment in &self.segments {
            let major_minor =
                size_async(self.horizontal, widthdb, segment, None, max_minor).await?;
            segments.push(Segment::new(major_minor, segment));
        }

        if let Some(available) = max_major {
            balance(&mut segments, available);

            let mut new_segments = Vec::with_capacity(self.segments.len());
            for (segment, balanced) in self.segments.iter().zip(segments.into_iter()) {
                let major_minor = size_async_with_balanced(
                    self.horizontal,
                    widthdb,
                    segment,
                    &balanced,
                    max_minor,
                )
                .await?;
                new_segments.push(Segment::new(major_minor, segment));
            }
            segments = new_segments;
        }

        let (major, minor) = sum_major_max_minor(&segments);
        let (width, height) = from_mm(self.horizontal, major, minor);
        Ok(Size::new(width, height))
    }

    async fn draw(self, frame: &mut Frame) -> Result<(), E> {
        let frame_size = frame.size();
        let (max_major, max_minor) = to_mm(self.horizontal, frame_size.width, frame_size.height);

        let widthdb = frame.widthdb();
        let mut segments = Vec::with_capacity(self.segments.len());
        for segment in &self.segments {
            let major_minor =
                size_async(self.horizontal, widthdb, segment, None, Some(max_minor)).await?;
            segments.push(Segment::new(major_minor, segment));
        }
        balance(&mut segments, max_major);

        let mut major = 0_i32;
        for (segment, balanced) in self.segments.into_iter().zip(segments.into_iter()) {
            let (x, y) = from_mm(self.horizontal, major, 0);
            let (w, h) = from_mm(self.horizontal, balanced.major, max_minor);
            frame.push(Pos::new(x, y), Size::new(w, h));
            segment.inner.draw(frame).await?;
            frame.pop();
            major += balanced.major as i32;
        }

        Ok(())
    }
}

macro_rules! mk_join {
    (
        pub struct $name:ident {
            $( pub $arg:ident: $type:ident [$n:expr], )+
        }
    ) => {
        pub struct $name< $($type),+ >{
            horizontal: bool,
            $( pub $arg: JoinSegment<$type>, )+
        }

        impl< $($type),+ > $name< $($type),+ >{
            pub fn horizontal( $($arg: JoinSegment<$type>),+ ) -> Self {
                Self { horizontal: true, $( $arg, )+ }
            }

            pub fn vertical( $($arg: JoinSegment<$type>),+ ) -> Self {
                Self { horizontal: false, $( $arg, )+ }
            }
        }

        impl<E, $($type),+ > Widget<E> for $name< $($type),+ >
        where
            $( $type: Widget<E>, )+
        {
            fn size(
                &self,
                widthdb: &mut WidthDb,
                max_width: Option<u16>,
                max_height: Option<u16>,
            ) -> Result<Size, E> {
                let (max_major, max_minor) = to_mm(self.horizontal, max_width, max_height);

                let mut segments = [ $(
                    Segment::new(
                        size(self.horizontal, widthdb, &self.$arg, None, max_minor)?,
                        &self.$arg,
                    ),
                )+ ];

                if let Some(available) = max_major {
                    balance(&mut segments, available);

                    let new_segments = [ $(
                        Segment::new(
                            size_with_balanced(self.horizontal, widthdb, &self.$arg, &segments[$n], max_minor)?,
                            &self.$arg,
                        ),
                    )+ ];
                    segments = new_segments;
                }

                let (major, minor) = sum_major_max_minor(&segments);
                let (width, height) = from_mm(self.horizontal, major, minor);
                Ok(Size::new(width, height))
            }

            #[allow(unused_assignments)]
            fn draw(self, frame: &mut Frame) -> Result<(), E> {
                let frame_size = frame.size();
                let (max_major, max_minor) = to_mm(self.horizontal, frame_size.width, frame_size.height);

                let widthdb = frame.widthdb();
                let mut segments = [ $(
                    Segment::new(
                        size(self.horizontal, widthdb, &self.$arg, None, Some(max_minor))?,
                        &self.$arg,
                    ),
                )+ ];
                balance(&mut segments, max_major);

                let mut major = 0_i32;
                $( {
                    let balanced = &segments[$n];
                    let (x, y) = from_mm(self.horizontal, major, 0);
                    let (w, h) = from_mm(self.horizontal, balanced.major, max_minor);
                    frame.push(Pos::new(x, y), Size::new(w, h));
                    self.$arg.inner.draw(frame)?;
                    frame.pop();
                    major += balanced.major as i32;
                } )*

                Ok(())
            }
        }

        #[async_trait]
        impl<E, $($type),+ > AsyncWidget<E> for $name< $($type),+ >
        where
            E: Send,
            $( $type: AsyncWidget<E> + Send + Sync, )+
        {
            async fn size(
                &self,
                widthdb: &mut WidthDb,
                max_width: Option<u16>,
                max_height: Option<u16>,
            ) -> Result<Size, E> {
                let (max_major, max_minor) = to_mm(self.horizontal, max_width, max_height);

                let mut segments = [ $(
                    Segment::new(
                        size_async(self.horizontal, widthdb, &self.$arg, None, max_minor).await?,
                        &self.$arg,
                    ),
                )+ ];

                if let Some(available) = max_major {
                    balance(&mut segments, available);

                    let new_segments = [ $(
                        Segment::new(
                            size_async_with_balanced(self.horizontal, widthdb, &self.$arg, &segments[$n], max_minor).await?,
                            &self.$arg,
                        ),
                    )+ ];
                    segments = new_segments;
                }

                let (major, minor) = sum_major_max_minor(&segments);
                let (width, height) = from_mm(self.horizontal, major, minor);
                Ok(Size::new(width, height))
            }

            #[allow(unused_assignments)]
            async fn draw(self, frame: &mut Frame) -> Result<(), E> {
                let frame_size = frame.size();
                let (max_major, max_minor) = to_mm(self.horizontal, frame_size.width, frame_size.height);

                let widthdb = frame.widthdb();
                let mut segments = [ $(
                    Segment::new(
                        size_async(self.horizontal, widthdb, &self.$arg, None, Some(max_minor)).await?,
                        &self.$arg,
                    ),
                )+ ];
                balance(&mut segments, max_major);

                let mut major = 0_i32;
                $( {
                    let balanced = &segments[$n];
                    let (x, y) = from_mm(self.horizontal, major, 0);
                    let (w, h) = from_mm(self.horizontal, balanced.major, max_minor);
                    frame.push(Pos::new(x, y), Size::new(w, h));
                    self.$arg.inner.draw(frame).await?;
                    frame.pop();
                    major += balanced.major as i32;
                } )*

                Ok(())
            }
        }
    };
}

mk_join! {
    pub struct Join2 {
        pub first: I1 [0],
        pub second: I2 [1],
    }
}

mk_join! {
    pub struct Join3 {
        pub first: I1 [0],
        pub second: I2 [1],
        pub third: I3 [2],
    }
}

mk_join! {
    pub struct Join4 {
        pub first: I1 [0],
        pub second: I2 [1],
        pub third: I3 [2],
        pub fourth: I4 [3],
    }
}

mk_join! {
    pub struct Join5 {
        pub first: I1 [0],
        pub second: I2 [1],
        pub third: I3 [2],
        pub fourth: I4 [3],
        pub fifth: I5 [4],
    }
}

mk_join! {
    pub struct Join6 {
        pub first: I1 [0],
        pub second: I2 [1],
        pub third: I3 [2],
        pub fourth: I4 [3],
        pub fifth: I5 [4],
        pub sixth: I6 [5],
    }
}

mk_join! {
    pub struct Join7 {
        pub first: I1 [0],
        pub second: I2 [1],
        pub third: I3 [2],
        pub fourth: I4 [3],
        pub fifth: I5 [4],
        pub sixth: I6 [5],
        pub seventh: I7 [6],
    }
}
