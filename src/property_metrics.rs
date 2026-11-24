use property_metrics::{InfraiClient, InspectionReminders, MaintenanceRequest, PropertySnapshot, TenantDocuments};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

fn block_on<F: Future>(mut future: F) -> F::Output {
    fn no_op(_: *const ()) {}
    fn clone(_: *const ()) -> RawWaker { RawWaker::new(std::ptr::null(), &VTABLE) }
    static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, no_op, no_op, no_op);
    let waker = unsafe { Waker::from_raw(RawWaker::new(std::ptr::null(), &VTABLE)) };
    let mut context = Context::from_waker(&waker);
    let mut future = unsafe { Pin::new_unchecked(&mut future) };
    loop { if let Poll::Ready(value) = future.as_mut().poll(&mut context) { return value; } }
}

fn main() {
    let snapshot = PropertySnapshot {
        maintenance: MaintenanceRequest { open: 8, urgent: 2 },
        documents: TenantDocuments { expiring: 3 },
        inspections: InspectionReminders { due: 4 },
    };
    let client = InfraiClient::from_env().expect("set INFRAI_API_KEY");
    let metrics = block_on(client.metrics_report(&snapshot)).expect("metrics envelope was rejected");
    let usage = block_on(client.usage_timeseries()).expect("usage envelope was rejected");
    println!("business metrics: {metrics}");
    println!("platform usage: {usage}");
    println!("The same key and base URL serve both series.");
}
