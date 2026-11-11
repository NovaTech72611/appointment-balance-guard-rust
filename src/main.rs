mod appointment_service;

use appointment_balance_guard::{Appointment, AppointmentState};
use appointment_service::InfraiClient;

fn block_on<T>(future: impl std::future::Future<Output = T>) -> T {
    use std::future::Future;
    use std::pin::Pin;
    use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
    fn no_op(_: *const ()) {}
    fn clone(_: *const ()) -> RawWaker {
        RawWaker::new(std::ptr::null(), &VTABLE)
    }
    static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, no_op, no_op, no_op);
    let waker = unsafe { Waker::from_raw(RawWaker::new(std::ptr::null(), &VTABLE)) };
    let mut context = Context::from_waker(&waker);
    let mut future = Box::pin(future);
    loop {
        if let Poll::Ready(value) = Future::poll(Pin::as_mut(&mut future), &mut context) {
            return value;
        }
    }
}

fn main() {
    let recipient = match std::env::var("APPOINTMENT_RECIPIENT") {
        Ok(recipient) if !recipient.is_empty() => recipient,
        _ => {
            eprintln!("APPOINTMENT_RECIPIENT is required");
            std::process::exit(1);
        }
    };
    let appointment = Appointment {
        id: "apt-demo".to_owned(),
        patient_email: recipient,
        state: AppointmentState::Confirmed,
    };
    let client = match InfraiClient::from_env() {
        Ok(client) => client,
        Err(error) => {
            eprintln!("{error:?}");
            std::process::exit(1);
        }
    };
    let result = block_on(async {
        client.configure_balance_guard().await?;
        client.balance().await?;
        client.send_patient_notice(&appointment).await
    });
    match result {
        Ok(value) => println!("workflow result: {value:?}"),
        Err(error) => {
            eprintln!("workflow error: {error:?}");
            std::process::exit(1);
        }
    }
}
