use std::ops::Deref;

pub enum Either<A, B> {
    Left(A),
    Right(B),
}

impl<A, B> Deref for Either<A, B>
where
    A: Deref,
    B: Deref<Target = A::Target>,
{
    type Target = A::Target;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Left(a) => a,
            Self::Right(b) => b,
        }
    }
}
