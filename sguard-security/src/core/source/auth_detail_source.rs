pub trait AuthDetailSource<T: AuthDetail, C> {
    fn build_detail(context: C) -> T;
}

pub trait AuthDetail {}
