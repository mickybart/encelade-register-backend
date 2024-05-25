pub(crate) enum TimeTraceFor {
    ClientInsideForCollect,
    ClientOutsideAfterCollect,
    ClientInsideForReturn,
    ClientOutsideAfterReturn,
}

pub(crate) enum SignatureTraceFor {
    CollectByClient,
    CollectConfirmedByPqrs,
    ReturnByClient,
    ReturnConfirmedByPqrs,
}