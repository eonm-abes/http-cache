use async_trait::async_trait;
use dyn_clonable::*;

/// Defines allowed transitions between states.
pub trait TransitionTo<S> {}

impl<S: ResourceState> Transition<S> {
    #[allow(clippy::boxed_local)]
    pub fn next<I: State<S>, O: State<S>>(_i: Box<I>, o: O) -> Transition<S>
    where
        I: TransitionTo<O>,
    {
        Transition::Next(StateHolder { state: Box::new(o) })
    }
}

pub trait ResourceState: 'static + Sync + Send {
    type Status;
}

#[clonable]
#[async_trait]
pub trait State<S: ResourceState>: Sync + Send + 'static + std::fmt::Debug + Clone {
    async fn next(self: Box<Self>, state: &mut S) -> Transition<S>;
}

pub enum Transition<S: ResourceState> {
    /// Transition to new state.
    Next(StateHolder<S>),
    /// Stop executing the state machine and report the result of the execution.
    Complete(Result<(), Box<dyn std::error::Error>>),
}

pub struct StateHolder<S: ResourceState> {
    pub state: Box<dyn State<S>>,
}

/// Defines allowed transitions between states.
#[macro_export]
macro_rules! transitions {
    ($from:ident => $to:ident) => {
        #[derive(Debug, Clone)]
        pub struct $from;

        #[derive(Debug, Clone)]
        pub struct $to;

        impl TransitionTo<$to> for $from {}
    };

    // State ending with a `*` is the terminal state.
    ($from:ident => $to:ident*) => {
        #[derive(Debug, Clone)]
        pub struct $to;

        impl TransitionTo<$to> for $from {}
    };
}
