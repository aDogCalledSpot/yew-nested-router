use crate::router::RouterContext;
use crate::target::{Mapper, Target};
use gloo_history::query::ToQuery;
use yew::prelude::*;
use super::router::ChangeTargetArgs;

#[derive(Clone, Debug, PartialEq)]
pub struct ScopeContext<C>
where
    C: Target,
{
    pub(crate) upwards: Callback<ChangeTargetArgs<C>>,
    pub(crate) collect: Callback<ChangeTargetArgs<C>, String>,
}

impl<C> ScopeContext<C>
where
    C: Target,
{
    pub(crate) fn push(&self, target: C) {
        self.upwards.emit(ChangeTargetArgs { target, query: None });
    }

    pub(crate) fn push_with_query<Q: ToQuery>(&self, target: C, query: Q) -> Result<(), Q::Error> {
        let query = query.to_query()?;
        let query = Some(query.into_owned());
        self.upwards.emit(ChangeTargetArgs { target, query });
        Ok(())
    }

    pub(crate) fn collect(&self, target: ChangeTargetArgs<C>) -> String {
        self.collect.emit(target)
    }
}

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct ScopeProps<P, C>
where
    P: Target,
    C: Target,
{
    pub mapper: Callback<(), Mapper<P, C>>,

    #[prop_or_default]
    pub children: Children,
}

/// A component, translating down to the next level.
#[function_component(Scope)]
pub fn scope<P, C>(props: &ScopeProps<P, C>) -> Html
where
    P: Target + 'static,
    C: Target + 'static,
{
    let router = use_context::<RouterContext<P>>()
        .expect("Must be nested under a Router or Nested component of the parent type");

    let parent = use_context::<ScopeContext<P>>()
        .expect("Must be nested under a Router or Nested component of the parent type");

    let Mapper { downwards, upwards } = props.mapper.emit(());

    let scope = use_memo((parent.clone(), upwards), |(parent, upwards)| {
        ScopeContext {
            upwards: {
                let parent = parent.upwards.clone();
                let upwards = upwards.clone();
                Callback::from(move |child: ChangeTargetArgs<C>| {
                    parent.emit(upwards.emit(child));
                })
            },
            collect: {
                let parent = parent.collect.clone();
                let upwards = upwards.clone();
                Callback::from(move |child: ChangeTargetArgs<C>| parent.emit(upwards.emit(child)))
            },
        }
    });

    let base = router.base.clone();
    let active = router.active();

    let context = use_memo(
        (
            base,
            scope.clone(),
            active.clone().and_then(|p| downwards.emit(p)),
        ),
        |(base, scope, target)| RouterContext {
            base: base.clone(),
            scope: scope.clone(),
            active_target: target.clone(),
        },
    );

    html!(
        <ContextProvider<RouterContext<C>> context={(*context).clone()}>
            <ContextProvider<ScopeContext<C>> context={(*scope).clone()}>
                { for props.children.iter() }
            </ContextProvider<ScopeContext<C>>>
        </ContextProvider<RouterContext<C>>>
    )
}
