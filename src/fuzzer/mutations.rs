use std::marker::PhantomData;

use libafl::{
    bolts::{
        rands::Rand,
        tuples::{tuple_list, tuple_list_type, Named},
    },
    corpus::Corpus,
    mutators::{MutationResult, Mutator},
    state::{HasCorpus, HasMaxSize, HasMetadata, HasRand},
    Error,
};

use crate::fuzzer::mutations_util::*;
use crate::term::Term;
use crate::tls::SIGNATURE;
use crate::trace::Trace;

pub fn trace_mutations<R, C, S>() -> tuple_list_type!(
       RepeatMutator<R, S>,
       SkipMutator<R, S>,
       ReplaceReuseMutator<R, S>,
       ReplaceMatchMutator<R, S>,
       RemoveAndLiftMutator<R, S>,
   )
where
    S: HasCorpus<C, Trace> + HasMetadata + HasMaxSize + HasRand<R>,
    C: Corpus<Trace>,
    R: Rand,
{
    tuple_list!(
        RepeatMutator::new(),
        SkipMutator::new(),
        ReplaceReuseMutator::new(),
        ReplaceMatchMutator::new(),
        RemoveAndLiftMutator::new(),
    )
}

/// REPEAT: Repeats an input which is already part of the trace
#[derive(Default)]
pub struct RepeatMutator<R, S>
where
    S: HasRand<R>,
    R: Rand,
{
    phantom: PhantomData<(R, S)>,
}

impl<R, S> Mutator<Trace, S> for RepeatMutator<R, S>
where
    S: HasRand<R>,
    R: Rand,
{
    fn mutate(
        &mut self,
        state: &mut S,
        trace: &mut Trace,
        _stage_idx: i32,
    ) -> Result<MutationResult, Error> {
        let steps = &trace.steps;
        let length = steps.len();
        if length == 0 {
            return Ok(MutationResult::Skipped);
        }
        let insert_index = state.rand_mut().between(0, (length - 1) as u64) as usize;
        let step = state.rand_mut().choose(steps).clone();
        (&mut trace.steps).insert(insert_index, step);
        Ok(MutationResult::Mutated)
    }
}

impl<R, S> Named for RepeatMutator<R, S>
where
    S: HasRand<R>,
    R: Rand,
{
    fn name(&self) -> &str {
        "RepeatMutator"
    }
}

impl<R, S> RepeatMutator<R, S>
where
    S: HasRand<R>,
    R: Rand,
{
    #[must_use]
    pub fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

/// SKIP:  Removes an input step
#[derive(Default)]
pub struct SkipMutator<R, S>
where
    S: HasRand<R>,
    R: Rand,
{
    phantom: PhantomData<(R, S)>,
}

impl<R, S> Mutator<Trace, S> for SkipMutator<R, S>
where
    S: HasRand<R>,
    R: Rand,
{
    fn mutate(
        &mut self,
        state: &mut S,
        trace: &mut Trace,
        _stage_idx: i32,
    ) -> Result<MutationResult, Error> {
        let steps = &mut trace.steps;
        let length = steps.len();
        if length == 0 {
            return Ok(MutationResult::Skipped);
        }
        let remove_index = state.rand_mut().between(0, (length - 1) as u64) as usize;
        steps.remove(remove_index);
        Ok(MutationResult::Mutated)
    }
}

impl<R, S> Named for SkipMutator<R, S>
where
    S: HasRand<R>,
    R: Rand,
{
    fn name(&self) -> &str {
        "SkipMutator"
    }
}

impl<R, S> SkipMutator<R, S>
where
    S: HasRand<R>,
    R: Rand,
{
    #[must_use]
    pub fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

/// REPLACE-REUSE: Replaces a sub-term with a different sub-term which is part of the trace
/// (such that types match). The new sub-term could come from another step which has a different recipe term.
#[derive(Default)]
pub struct ReplaceReuseMutator<R, S>
where
    S: HasRand<R>,
    R: Rand,
{
    phantom: PhantomData<(R, S)>,
}

impl<R, S> ReplaceReuseMutator<R, S>
where
    S: HasRand<R>,
    R: Rand,
{
    #[must_use]
    pub fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl<R, S> Mutator<Trace, S> for ReplaceReuseMutator<R, S>
where
    S: HasRand<R>,
    R: Rand,
{
    // todo make sure that we do not replace a term with itself (performance improvement)
    fn mutate(
        &mut self,
        state: &mut S,
        trace: &mut Trace,
        _stage_idx: i32,
    ) -> Result<MutationResult, Error> {
        let rand = state.rand_mut();
        if let Some(replacement) = choose_term(trace, rand).cloned() {
            if let Some(to_replace) = choose_term_mut(trace, rand, |term: &Term| {
                term.get_type_shape() == replacement.get_type_shape()
            }) {
                to_replace.mutate(&replacement);
                return Ok(MutationResult::Mutated);
            }
        }

        Ok(MutationResult::Skipped)
    }
}

impl<R, S> Named for ReplaceReuseMutator<R, S>
where
    S: HasRand<R>,
    R: Rand,
{
    fn name(&self) -> &str {
        "ReplaceReuseMutator"
    }
}

/// REPLACE-MATCH: Replaces a function symbol with a different one (such that types match).
/// An example would be to replace a constant with another constant or the binary function
/// fn_add with fn_sub.
#[derive(Default)]
pub struct ReplaceMatchMutator<R, S>
where
    S: HasRand<R>,
    R: Rand,
{
    phantom: PhantomData<(R, S)>,
}

impl<R, S> ReplaceMatchMutator<R, S>
where
    S: HasRand<R>,
    R: Rand,
{
    #[must_use]
    pub fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl<R, S> Mutator<Trace, S> for ReplaceMatchMutator<R, S>
where
    S: HasRand<R>,
    R: Rand,
{
    fn mutate(
        &mut self,
        state: &mut S,
        trace: &mut Trace,
        _stage_idx: i32,
    ) -> Result<MutationResult, Error> {
        let rand = state.rand_mut();
        let (requested_shape, requested_dynamic_fn) = rand.choose(&SIGNATURE.functions);

        let filter = |term: &Term| match term {
            Term::Variable(_) => false,
            Term::Application(func, _) => {
                func.shape().name != requested_shape.name
                    && func.shape().return_type == requested_shape.return_type
                    && func.shape().argument_types == requested_shape.argument_types
            }
        };
        if let Some(mut to_mutate) = choose_term_mut(trace, rand, filter) {
            match &mut to_mutate {
                Term::Variable(_) => {
                    // never reached as `filter` returns false for variables
                    Ok(MutationResult::Skipped)
                }
                Term::Application(func, _) => {
                    func.change_function(requested_shape.clone(), requested_dynamic_fn.clone());
                    Ok(MutationResult::Mutated)
                }
            }
        } else {
            Ok(MutationResult::Skipped)
        }
    }
}

impl<R, S> Named for ReplaceMatchMutator<R, S>
where
    S: HasRand<R>,
    R: Rand,
{
    fn name(&self) -> &str {
        "ReplaceMatchMutator"
    }
}

/// REMOVE AND LIFT: Removes a sub-term from a term and attaches orphaned children to the parent
/// (such that types match). This only works if there is only a single child.
#[derive(Default)]
pub struct RemoveAndLiftMutator<R, S>
where
    S: HasRand<R>,
    R: Rand,
{
    phantom: PhantomData<(R, S)>,
}

impl<R, S> RemoveAndLiftMutator<R, S>
where
    S: HasRand<R>,
    R: Rand,
{
    #[must_use]
    pub fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl<R, S> Mutator<Trace, S> for RemoveAndLiftMutator<R, S>
where
    S: HasRand<R>,
    R: Rand,
{
    fn mutate(
        &mut self,
        state: &mut S,
        trace: &mut Trace,
        _stage_idx: i32,
    ) -> Result<MutationResult, Error> {
        let rand = state.rand_mut();

        // filter for inner nodes with exactly one subterm
        let filter = |term: &Term| match term {
            Term::Variable(_) => false,
            Term::Application(func, _) => {
                func.shape().argument_types.len() == 1
                    && func.shape().argument_types.first().unwrap() == &func.shape().return_type
            }
        };
        if let Some(mut to_mutate) = choose_term_mut(trace, rand, filter) {
            match &to_mutate {
                Term::Variable(_) => {
                    // never reached as `filter` returns false for variables
                    Ok(MutationResult::Skipped)
                }
                Term::Application(_func, subterms) => {
                    let subterm = subterms.clone();
                    to_mutate.mutate(subterm.first().unwrap());
                    Ok(MutationResult::Mutated)
                }
            }
        } else {
            Ok(MutationResult::Skipped)
        }
    }
}

impl<R, S> Named for RemoveAndLiftMutator<R, S>
where
    S: HasRand<R>,
    R: Rand,
{
    fn name(&self) -> &str {
        "RemoveAndLiftMutator"
    }
}

// todo SWAP: https://github.com/Sgeo/take_mut
