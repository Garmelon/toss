use async_trait::async_trait;

use crate::{AsyncWidget, Frame, Size, Widget, WidthDb};

#[derive(Debug, Clone)]
pub struct Layer<I> {
    layers: Vec<I>,
}

impl<I> Layer<I> {
    pub fn new(layers: Vec<I>) -> Self {
        Self { layers }
    }
}

impl<E, I> Widget<E> for Layer<I>
where
    I: Widget<E>,
{
    fn size(
        &self,
        widthdb: &mut WidthDb,
        max_width: Option<u16>,
        max_height: Option<u16>,
    ) -> Result<Size, E> {
        let mut size = Size::ZERO;
        for layer in &self.layers {
            let lsize = layer.size(widthdb, max_width, max_height)?;
            size.width = size.width.max(lsize.width);
            size.height = size.height.max(lsize.height);
        }
        Ok(size)
    }

    fn draw(self, frame: &mut Frame) -> Result<(), E> {
        for layer in self.layers {
            layer.draw(frame)?;
        }
        Ok(())
    }
}

#[async_trait]
impl<E, I> AsyncWidget<E> for Layer<I>
where
    I: AsyncWidget<E> + Send + Sync,
{
    async fn size(
        &self,
        widthdb: &mut WidthDb,
        max_width: Option<u16>,
        max_height: Option<u16>,
    ) -> Result<Size, E> {
        let mut size = Size::ZERO;
        for layer in &self.layers {
            let lsize = layer.size(widthdb, max_width, max_height).await?;
            size.width = size.width.max(lsize.width);
            size.height = size.height.max(lsize.height);
        }
        Ok(size)
    }

    async fn draw(self, frame: &mut Frame) -> Result<(), E> {
        for layer in self.layers {
            layer.draw(frame).await?;
        }
        Ok(())
    }
}

macro_rules! mk_layer {
    (
        pub struct $name:ident {
            $( pub $arg:ident: $type:ident, )+
        }
    ) => {
        #[derive(Debug, Clone, Copy)]
        pub struct $name< $($type),+ >{
            $( pub $arg: $type, )+
        }

        impl< $($type),+ > $name< $($type),+ >{
            pub fn new( $($arg: $type),+ ) -> Self {
                Self { $( $arg, )+ }
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
                let mut size = Size::ZERO;

                $({
                    let lsize = self.$arg.size(widthdb, max_width, max_height)?;
                    size.width = size.width.max(lsize.width);
                    size.height = size.height.max(lsize.height);
                })+

                Ok(size)
            }

            fn draw(self, frame: &mut Frame) -> Result<(), E> {
                $( self.$arg.draw(frame)?; )+
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
                let mut size = Size::ZERO;

                $({
                    let lsize = self.$arg.size(widthdb, max_width, max_height).await?;
                    size.width = size.width.max(lsize.width);
                    size.height = size.height.max(lsize.height);
                })+

                Ok(size)
            }

            async fn draw(self, frame: &mut Frame) -> Result<(), E> {
                $( self.$arg.draw(frame).await?; )+
                Ok(())
            }
        }
    };
}

mk_layer!(
    pub struct Layer2 {
        pub first: I1,
        pub second: I2,
    }
);

mk_layer!(
    pub struct Layer3 {
        pub first: I1,
        pub second: I2,
        pub third: I3,
    }
);

mk_layer!(
    pub struct Layer4 {
        pub first: I1,
        pub second: I2,
        pub third: I3,
        pub fourth: I4,
    }
);

mk_layer!(
    pub struct Layer5 {
        pub first: I1,
        pub second: I2,
        pub third: I3,
        pub fourth: I4,
        pub fifth: I5,
    }
);

mk_layer!(
    pub struct Layer6 {
        pub first: I1,
        pub second: I2,
        pub third: I3,
        pub fourth: I4,
        pub fifth: I5,
        pub sixth: I6,
    }
);

mk_layer!(
    pub struct Layer7 {
        pub first: I1,
        pub second: I2,
        pub third: I3,
        pub fourth: I4,
        pub fifth: I5,
        pub sixth: I6,
        pub seventh: I7,
    }
);
