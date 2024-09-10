use crate::algebra::{Matcher, Term};
use crate::execution::ExecutionStatus;
use crate::graphviz::write_graphviz;
use crate::trace::{Action, Trace};

impl<M: Matcher> Trace<M> {
    pub fn count_functions_by_name(&self, find_name: &'static str) -> usize {
        self.steps
            .iter()
            .map(|step| match &step.action {
                Action::Input(input) => input.recipe.count_functions_by_name(find_name),
                Action::Output(_) => 0,
            })
            .sum()
    }

    pub fn count_functions(&self) -> usize {
        self.steps
            .iter()
            .flat_map(|step| match &step.action {
                Action::Input(input) => Some(&input.recipe),
                Action::Output(_) => None,
            })
            .map(|term| term.size())
            .sum()
    }

    pub fn write_plots(&self, i: u16) {
        write_graphviz(
            format!("test_mutation{}.svg", i).as_str(),
            "svg",
            self.dot_graph(true).as_str(),
        )
        .unwrap();
    }
}

impl<M: Matcher> Term<M> {
    pub fn count_functions_by_name(&self, find_name: &'static str) -> usize {
        let mut found = 0;
        for term in self.into_iter() {
            if let Term::Application(func, _) = term {
                if func.name() == find_name {
                    found += 1;
                }
            }
        }
        found
    }
}

pub trait AssertExecution {
    fn expect_crash(self);
}

impl AssertExecution for Result<ExecutionStatus, String> {
    fn expect_crash(self) {
        use ExecutionStatus as S;
        match self {
            Ok(S::Failure(_)) | Ok(S::Crashed) => (),
            Ok(S::Timeout) => panic!("trace execution timed out"),
            Ok(S::Success) => panic!("expected trace execution to crash, but succeeded"),
            Err(reason) => panic!("trace execution error: {reason}"),
        }
    }
}
