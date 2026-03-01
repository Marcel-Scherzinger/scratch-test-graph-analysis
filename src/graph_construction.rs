use std::collections::HashMap;

use either::Either;
use smodel::{
    attrs::{DoForAttrs, DoForAttrsStrategy, Expression, List, ProcedureArgumentId},
    blocks::BlockWrapper,
};

struct Graph<'a> {
    phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> DoForAttrsStrategy<'a> for Graph<'a> {
    type Inputs = ();
    type Outputs = (Vec<&'a smodel::Id>, Vec<&'a List>);
    type Error = std::convert::Infallible;
}

pub(crate) fn add_edges_from_block<'a>(
    parameter_edges: &mut HashMap<&'a smodel::Id, Vec<&'a smodel::Id>>,
    read_list_edges: &mut HashMap<&'a smodel::Id, Vec<&'a List>>,
    next_block_edges: &mut HashMap<&'a smodel::Id, Option<&'a smodel::Id>>,
    parent_block_edges: &mut HashMap<&'a smodel::Id, Option<&'a smodel::Id>>,
    from_block: &'a BlockWrapper,
) -> Option<()> {
    let id = from_block.id();
    next_block_edges.insert(id, from_block.next().as_ref());
    parent_block_edges.insert(id, from_block.parent().as_ref());

    let mut both = (vec![], vec![]);

    DoForAttrs::<'a, Graph<'a>>::do_for_attrs(from_block.inner(), &(), &mut both);

    let p = if both.0.is_empty() {
        false
    } else {
        parameter_edges.insert(id, both.0).is_some()
    };

    let r = if both.1.is_empty() {
        false
    } else {
        read_list_edges.insert(id, both.1).is_some()
    };

    if p || r { None } else { Some(()) }
}

impl<'a, T> DoForAttrs<'a, Graph<'a>> for smodel::attrs::RefBlock<T> {
    fn do_for_attrs(
        &'a self,
        _inputs: &<Graph<'a> as DoForAttrsStrategy<'a>>::Inputs,
        (outputs, _): &mut <Graph<'a> as DoForAttrsStrategy<'a>>::Outputs,
    ) -> Result<(), <Graph<'a> as DoForAttrsStrategy<'a>>::Error> {
        outputs.push(self.id());
        Ok(())
    }
}

impl<'a, T> DoForAttrs<'a, Graph<'a>> for Option<smodel::attrs::RefBlock<T>> {
    fn do_for_attrs(
        &'a self,
        _inputs: &<Graph<'a> as DoForAttrsStrategy<'a>>::Inputs,
        (outputs, _): &mut <Graph<'a> as DoForAttrsStrategy<'a>>::Outputs,
    ) -> Result<(), <Graph<'a> as DoForAttrsStrategy<'a>>::Error> {
        if let Some(id) = self.as_ref() {
            outputs.push(id.id());
        }
        Ok(())
    }
}
impl<'a> DoForAttrs<'a, Graph<'a>> for HashMap<ProcedureArgumentId, Option<attrs::Expression>> {
    fn do_for_attrs(
        &'a self,
        inputs: &<Graph<'a> as DoForAttrsStrategy<'a>>::Inputs,
        outputs: &mut <Graph<'a> as DoForAttrsStrategy<'a>>::Outputs,
    ) -> Result<(), <Graph<'a> as DoForAttrsStrategy<'a>>::Error> {
        for o in self.values().flatten() {
            o.do_for_attrs(inputs, outputs)?;
        }
        Ok(())
    }
}

impl<'a, T> DoForAttrs<'a, Graph<'a>> for Either<T, attrs::ExpressionRef>
where
    T: DoForAttrs<'a, Graph<'a>>,
{
    fn do_for_attrs(
        &'a self,
        inputs: &<Graph<'a> as DoForAttrsStrategy<'a>>::Inputs,
        outputs: &mut <Graph<'a> as DoForAttrsStrategy<'a>>::Outputs,
    ) -> Result<(), <Graph<'a> as DoForAttrsStrategy<'a>>::Error> {
        match self {
            Either::Left(l) => l.do_for_attrs(inputs, outputs),
            Either::Right(r) => r.do_for_attrs(inputs, outputs),
        }
    }
}

impl<'a> DoForAttrs<'a, Graph<'a>> for attrs::Expression {
    fn do_for_attrs(
        &'a self,
        inputs: &<Graph<'a> as DoForAttrsStrategy<'a>>::Inputs,
        outputs: &mut <Graph<'a> as DoForAttrsStrategy<'a>>::Outputs,
    ) -> Result<(), <Graph<'a> as DoForAttrsStrategy<'a>>::Error> {
        match self {
            Expression::Blo(block) => block.do_for_attrs(inputs, outputs)?,
            Expression::Var(_) => {}
            Expression::Lit(_) => {}
            Expression::Lis(list) => {
                outputs.1.push(list);
            }
        }

        Ok(())
    }
}

macro_rules! noop_do_impl {
    ($life: lifetime, $strat: ty {
        $(
        $({$($T: ident),*})? $ty: ty
    ),* $(,)?
    }) => {
        $(
        impl<$life, $($($T),*)?> DoForAttrs<'a, $strat> for $ty {
            fn do_for_attrs(
                &'a self,
                _inputs: &<$strat as DoForAttrsStrategy<'a>>::Inputs,
                _outputs: &mut <$strat as DoForAttrsStrategy<'a>>::Outputs,
            ) -> Result<(), <$strat as DoForAttrsStrategy<'a>>::Error> {
                Ok(())
            }
        }
        )*
    };
}

use smodel::attrs;
noop_do_impl!('a, Graph<'a> {
    attrs::Color,
    {T} attrs::DirectDropdownOf<T>,
    {T} attrs::DropdownMenuOf<T>,
    attrs::List,
    attrs::Variable,
    attrs::BroadcastId,
    smodel::blocks::ProcedureId,
    bool,
    attrs::ArgumentReporterName,
    svalue::ARc<[smodel::blocks::ProcedureArgumentDef]>,
    svalue::ARc<str>,
});
