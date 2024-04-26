pub enum TimeTraceFor {
    ClientInsideForCollect,
    ClientOutsideAfterCollect,
    ClientInsideForReturn,
    ClientOutsideAfterReturn,
}

pub enum SignatureTraceFor {
    CollectByClient,
    CollectConfirmedByPqrs,
    ReturnByClient,
    ReturnConfirmedByPqrs,
}