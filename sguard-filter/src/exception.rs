use std::{future::Future, pin::Pin, sync::Arc};

use hyper::{Body, Error, Request, Response};

use crate::core::{Filter, FilterFn, FilterRs};
use crate::filter_chain::FilterChainTrait;

pub trait ExceptionTranslationFilterTrait: FilterChainTrait{
    fn sub_filter_chain(&self) -> Option<Arc<dyn ExceptionTranslationFilterTrait>>;
}
pub struct ExceptionTranslationFilter {
    sub_chain: Option<Arc<dyn ExceptionTranslationFilterTrait>>,
}
impl ExceptionTranslationFilter {
    pub fn new(sub_chain: Option<Arc<dyn ExceptionTranslationFilterTrait>>) -> Self {
        ExceptionTranslationFilter { sub_chain }
    }
}
impl Filter for ExceptionTranslationFilter {
    fn handle(&self, req: &Request<Body>, next: FilterFn) -> FilterRs {
        log::debug!("Filter: ExceptionTranslationFilter");
        next(req)
    }

    fn sub_filter_chain(&self) -> Option<Arc<dyn Filter>> {
        todo!()
    }
}
