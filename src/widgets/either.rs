use async_trait::async_trait;

use crate::{AsyncWidget, Frame, Size, Widget};

macro_rules! mk_either {
    (
        pub enum $name:ident {
            $( $constr:ident($ty:ident), )+
        }
    ) => {
        #[derive(Debug, Clone, Copy)]
        pub enum $name< $( $ty ),+ > {
            $( $constr($ty), )+
        }

        impl<E, $( $ty ),+> Widget<E> for $name< $( $ty ),+ >
        where
            $( $ty: Widget<E>, )+
        {
            fn size(
                &self,
                frame: &mut Frame,
                max_width: Option<u16>,
                max_height: Option<u16>,
            ) -> Result<Size, E> {
                match self {
                    $( Self::$constr(w) => w.size(frame, max_width, max_height), )+
                }
            }

            fn draw(self, frame: &mut Frame) -> Result<(), E> {
                match self {
                    $( Self::$constr(w) => w.draw(frame), )+
                }
            }
        }

        #[async_trait]
        impl<E, $( $ty ),+> AsyncWidget<E> for $name< $( $ty ),+ >
        where
            $( $ty: AsyncWidget<E> + Send + Sync, )+
        {
            async fn size(
                &self,
                frame: &mut Frame,
                max_width: Option<u16>,
                max_height: Option<u16>,
            ) -> Result<Size, E> {
                match self {
                    $( Self::$constr(w) => w.size(frame, max_width, max_height).await, )+
                }
            }

            async fn draw(self, frame: &mut Frame) -> Result<(), E> {
                match self {
                    $( Self::$constr(w) => w.draw(frame).await, )+
                }
            }
        }
    };
}

mk_either! {
    pub enum Either2 {
        First(I1),
        Second(I2),
    }
}

mk_either! {
    pub enum Either3 {
        First(I1),
        Second(I2),
        Third(I3),
    }
}

mk_either! {
    pub enum Either4 {
        First(I1),
        Second(I2),
        Third(I3),
        Fourth(I4),
    }
}

mk_either! {
    pub enum Either5 {
        First(I1),
        Second(I2),
        Third(I3),
        Fourth(I4),
        Fifth(I5),
    }
}

mk_either! {
    pub enum Either6 {
        First(I1),
        Second(I2),
        Third(I3),
        Fourth(I4),
        Fifth(I5),
        Sixth(I6),
    }
}

mk_either! {
    pub enum Either7 {
        First(I1),
        Second(I2),
        Third(I3),
        Fourth(I4),
        Fifth(I5),
        Sixth(I6),
        Seventh(I7),
    }
}
